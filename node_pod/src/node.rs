use crate::{block_header::{Block, BlockChain, BlockMethods, BlockChainMethods}, transaction_header::{Transaction, TransactionMethods}};
use bls_signatures::{PrivateKey, PublicKey, Serialize};
use ring::signature::{UnparsedPublicKey, ED25519};
use futures_util::StreamExt;
use std::collections::HashMap;
use crate::consensus_header::Pbft;
use rdkafka::{consumer::{Consumer, StreamConsumer, BaseConsumer}, producer::{Producer,BaseProducer, BaseRecord}, ClientConfig};
use tokio::time::{timeout, Instant};
use crate::network::{consume_kafka, ready_state};
use crate::node_header::NodeMethods;
use rdkafka::Message;
use std::time::Duration;
use std::sync::Arc;
use crate::network_header::{Network, NodeMessage};
use tokio;
use crate::node_header::{Node, NodeState, PoolingMetrics, ConcensusMetrics};

// use serde_json::to_string;transaction_header


///         WORK ON CORDINATING THE CONCENSUS STEPS AND PROCESS

impl Network for Node {
    async fn broadcast_kafka(&self, topic: &str, message: NodeMessage, producer: &BaseProducer){
        let message = serde_json::to_string(&message).expect("Failed to parse Block");

        producer.send(
            BaseRecord::to(topic)
            .payload(&message)
            .key(&self.id)
        ).expect("Failed to send message");

        producer.flush(Duration::from_secs(20)).expect("Failed to flush");

        if topic == "Prepare" {
            producer.send(
                BaseRecord::to("Status")
                .payload("Commit")
                .key(&self.id)
            ).expect("Failed to send message");
        }

        producer.flush(Duration::from_secs(20)).expect("Failed to flush");
    }

    fn consume_kafka(&self, topic: &str) -> Vec<String>{
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("group.id", &(self.id.clone()+"consumer"))
            .set("enable.auto.commit","true")
            .create()
            .expect("Failed to make consumer");

        consumer.subscribe(&[topic]).expect("Subscription Error");

        let timeout = Duration::from_secs(10);
        let mut message_pool = Vec::new();
        let mut last_received_time = None;

        loop {
            // Wait for all the Nodes to broadcast the message
            // 5 seconds is the timeout after which it will move on to process the messages
            match consumer.poll(Duration::from_secs(5)) {
                Some(Ok(message)) => {
                    if let Some(payload) = message.payload() {
                        let msg = String::from(std::str::from_utf8(payload).expect("Failed to deserialize message"));
                        message_pool.push(msg);
                        last_received_time = Some(Instant::now());
                    }
                }
                _ => continue,
            }
            match last_received_time {
                Some(time) => {
                    if time.elapsed() > timeout {
                        break;
                    }
                }
                _ => { continue; }
            }
        }
        message_pool
    }
}

impl NodeMethods for Node {
    fn new() -> Self {
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

    fn sign_message(&self, block: &Block) -> String {
        let signature = self.private_key.sign(block.serialize_block());
        hex::encode(signature.as_bytes())
    }

    // Will have to change the pooling logic to pool only then wait

    async fn pool_transactions(&mut self, consumer: &StreamConsumer, user_base: &mut HashMap<String, f64>,
         residual: &mut Vec<Transaction>, time_out: u64, tx_time: u64, block_size: &usize) -> 
         (Option<Vec<Transaction>>,Vec<Transaction>, Option<PoolingMetrics>){

        let mut pool: Vec<Transaction> = Vec::with_capacity(*block_size);

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
                                                
                                                if balance < transaction.amount + transaction.fee {
                                                    continue;
                                                }

                                                let pub_key_bytes = hex::decode(&transaction.from).unwrap();
                                                let public_key = UnparsedPublicKey::new(&ED25519, pub_key_bytes);
                                                if transaction.verify_transaction(public_key) == true {
                                                    let new_balance = balance - transaction.amount - transaction.fee;
                                                    user_base.insert(transaction.from.clone(), new_balance);
                                                    pool.push(transaction.to_owned());
                                                } else { continue; }
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

    async fn concensus(&mut self, pool: Vec<Transaction>, pkey_store: HashMap<String, PublicKey>,
    prepre_con: Arc<StreamConsumer>, pre_con: Arc<StreamConsumer>,
    prepre_ready: &StreamConsumer, pre_ready: &StreamConsumer, commit_ready: &StreamConsumer,
    prepre_prod: Arc<BaseProducer>, pre_prod: Arc<BaseProducer>, comm_prod: Arc<BaseProducer>,
    time_out: u64) -> Option<ConcensusMetrics>{

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