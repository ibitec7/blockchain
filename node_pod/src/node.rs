use crate::{block::{Block, BlockChain}, transaction::Transaction};
use bls_signatures::{PrivateKey, PublicKey, Serialize};
use futures_util::StreamExt;
use std::collections::HashMap;
use crate::concensus::Pbft;
use rdkafka::{consumer::{Consumer, StreamConsumer}, producer::BaseProducer};
use tokio::time::{timeout, Instant};
use crate::network::{consume_kafka, ready_state};
use rdkafka::Message;
use std::time::Duration;
use std::sync::Arc;
use tokio;
// use serde_json::to_string;
use serde::{Deserialize, Serialize as SerdeSerialize};

use crate::concensus::Validator;

///         WORK ON CORDINATING THE CONCENSUS STEPS AND PROCESS

#[derive(SerdeSerialize, Deserialize, Clone, std::fmt::Debug)]
pub enum NodeState {
    Idle,
    PrePreparing,
    Preparing,
    Committing,
    Done
}

pub struct Node {
    pub id: String,
    pub block_chain: BlockChain,
    pub stake: f64,
    pub state: NodeState,
    pub staging: Vec<Transaction>,
    pub block_staging: Vec<Block>,
    pub validators: Vec<Validator>,
    pub primary: Vec<Validator>,
    pub msg_idx: Vec<usize>,
    private_key: PrivateKey
}

pub struct PoolingMetrics {
    pub tps: f64,
    pub processtime: f64,
    pub bad_tx: f64,
    pub ttf: f64
}

pub struct ConcensusMetrics {
    pub prepre_time: f64,
    pub pre_time: f64,
    pub commit_time: f64,
    pub prepre_wait: f64,
    pub pre_wait: f64,
    pub commit_wait: f64
}

impl Node {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let pvt_key = PrivateKey::generate(&mut rng);
        let pub_key = pvt_key.public_key();

        let id = hex::encode(pub_key.as_bytes());

        let indexes: Vec<usize> = vec![0,0,0];

        let node = Node { id: id, block_chain: BlockChain::new(), stake: 0.0, state: NodeState::Idle,
             staging: vec![], block_staging: vec![], validators: vec![],
            primary: vec![], msg_idx: indexes, private_key: pvt_key };
        node
    }

    pub fn sign_message(&self, block: &Block) -> String {
        let signature = self.private_key.sign(block.serialize_block());
        hex::encode(signature.as_bytes())
    }

    // Will have to change the pooling logic to pool only then wait

    pub async fn pool_transactions(&mut self, consumer: &StreamConsumer, user_base: &mut HashMap<String, f64>,
         residual: &mut Vec<Transaction>, time_out: u64, tx_time: u64, block_size: &usize) -> (Option<Vec<Transaction>>,Vec<Transaction>, Option<PoolingMetrics>){

        let mut pool: Vec<Transaction> = Vec::new();

        let mut message_stream = consumer.stream();

        let start = Instant::now();
        // let mut transactions: f64 = 0.0;

        loop {
            let ttf_start = Instant::now();
            match timeout(Duration::from_millis(time_out), message_stream.next()).await {
            Ok(Some(message_result)) => {
                match message_result {
                    Err(_) => panic!["Error while receiving message"],
                    Ok(message) => {
                            if let Some(payload) = message.payload() {
                                let transaction_result: Result<Vec<Transaction>, _> = serde_json::from_slice(payload);
                                match transaction_result{
                                    Ok(mut transaction_vec) => {
                                        tokio::time::sleep(Duration::from_millis(tx_time)).await;

                                        // transactions += 64.0;
                                        if let Err(e) = consumer.commit_message(&message, rdkafka::consumer::CommitMode::Sync) {
                                            eprintln!("Failed to commit message: {}", e);
                                        }

                                        residual.append(&mut transaction_vec);
                                        if residual.len() > *block_size {
                                            let s1 = Instant::now();
                                            let mut a: f64 = 0.0;
                                            for transaction in residual.clone() {
                                                a = a+1.0;    
                                                let balance = user_base.get(&transaction.from).unwrap().to_owned();
                                                if transaction.verify_transaction(PublicKey::from_bytes(
                                                    &hex::decode(&transaction.from).unwrap().as_slice()).unwrap()) == true
                                                && balance > (transaction.amount + transaction.fee) {
                                                    let new_balance = balance - transaction.amount - transaction.fee;
                                                    user_base.insert(transaction.from.clone(), new_balance);
                                                    pool.push(transaction.to_owned());
                                                } else { continue; }
                                                //once pool has been filled then return the pool
                                                if pool.len() == *block_size { 
                                                    let end = s1.elapsed().as_millis() as f64;
                                                    let tps = 1000.0 * (a / (start.elapsed().as_millis() as f64));
                                                    let ttf = ttf_start.elapsed().as_millis() as f64;
                                                    let bad_tx = a - (*block_size as f64);
                                                    let metrics = PoolingMetrics {
                                                        tps, processtime: end, bad_tx, ttf
                                                    };
                                                    return (Some(pool),residual.to_owned(), Some(metrics));
                                                } else { continue; }
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        panic!["Failed to deserialize message"];
                                    }
                                }
                            }
                        }
                    }
                }
            Ok(_) => {
                println!("Stream Ended..");
                break;
            }
            Err(_) => {
                if !pool.is_empty(){
                    break;
                } else {
                    continue;
                }
            }
            }
        }
        if pool.is_empty() {
            return (None,residual.to_owned(), None);
        }
        else {
            return (Some(pool),residual.to_owned(), None);
        }
    }

    pub async fn concensus(&mut self, pool: Vec<Transaction>, pkey_store: HashMap<String, PublicKey>,
    prepre_con: Arc<StreamConsumer>, pre_con: Arc<StreamConsumer>,
    prepre_ready: &StreamConsumer, pre_ready: &StreamConsumer, commit_ready: &StreamConsumer,
    prepre_prod: Arc<BaseProducer>, pre_prod: Arc<BaseProducer>, comm_prod: Arc<BaseProducer>,
    time_out: u64) -> Option<ConcensusMetrics> {

        let id = self.id.clone()+"pre";
        let id2 = self.id.clone()+"prep";
        let hash = self.msg_idx[0].clone();
        let hash2 = hash.clone();

        let prepre_con_clone = Arc::clone(&prepre_con);
        let prepre_prod_clone = Arc::clone(&prepre_prod);

        let start1 = Instant::now();

        let primary_handle = tokio::spawn(async move {
            consume_kafka(id, hash.to_string(),"Preprepare", &prepre_con_clone, &prepre_prod_clone, time_out).await
        });

        //  wait for the preprepare message from all nodes here

        // tokio::time::sleep(Duration::from_millis(time_out)).await;

        let (a, preprepare_wait) = ready_state(self.validators.len(), String::from("Preprepare"), prepre_ready, time_out).await;

        if a {self.preprepare_phase(pool, &prepre_prod).await;}
        else {panic!["returned false..."]}

        let primary_msg: Vec<String> = primary_handle.await.unwrap().unwrap();

        let end1 = start1.elapsed().as_millis() as f64;

        // wait for the prepare message from all nodes here
        let pre_con_clone = Arc::clone(&pre_con);
        let pre_prod_clone = Arc::clone(&pre_prod);

        let start2 = Instant::now();

        let prepare_handle = tokio::spawn(async move {
            consume_kafka(id2, hash2.to_string(),"Prepare", &pre_con_clone, &pre_prod_clone, time_out).await
        });

        // tokio::time::sleep(Duration::from_millis(time_out)).await;

        let (b, prepare_wait) = ready_state(self.validators.len(), String::from("Prepare"), pre_ready, time_out).await;

        if b {self.prepare_phase(&pkey_store, primary_msg, &pre_prod).await;} 
        else {panic!["returned false"];}

        let prepare_msg: Vec<String> = prepare_handle.await.unwrap().unwrap();

        if prepare_msg.is_empty() {
            panic!["No prepare message received"];
        }

        let end2 = start2.elapsed().as_millis() as f64;

        // wait for the commit message from all nodes here

        let start3 = Instant::now();

        // tokio::time::sleep(Duration::from_millis(time_out)).await;

        let (c, commit_wait) = ready_state(self.validators.len(), String::from("Commit"), commit_ready, time_out).await;

        if c {self.commit_phase(&pkey_store,prepare_msg, &comm_prod).await;}

        let end3 = start3.elapsed().as_millis() as f64;

       Some(ConcensusMetrics { prepre_time: end1, pre_time: end2, commit_time: end3,
         prepre_wait: preprepare_wait, pre_wait: prepare_wait, commit_wait })

    }
}

