use crate::{block::{Block, BlockChain}, concensus::{Pbft, PoS}, simulate::User, transaction::Transaction};
use bls_signatures::{PrivateKey, PublicKey, Serialize};
use rand::{distributions::Alphanumeric, Rng};

use rdkafka::{consumer::{CommitMode, Consumer, StreamConsumer}, ClientConfig};
use rdkafka::message::Message;
use futures_util::stream::StreamExt;
use serde_json::to_string;
use serde::{Deserialize, Serialize as SerdeSerialize};

#[derive(SerdeSerialize, Deserialize, Clone, std::fmt::Debug)]
pub enum NodeState {
    Idle,
    PrePreparing,
    Preparing,
    Committing,
    Done
}

#[derive(SerdeSerialize, Deserialize, Clone, std::fmt::Debug)]
pub struct Node {
    pub id: String,
    pub block_chain: BlockChain,
    pub stake: f64,
    pub state: NodeState,
    pub staging: Vec<Transaction>,
    pub validators: Vec<String>,
    pub primary: Vec<String>,
}

#[derive(Clone)]
pub struct Miner {
    pub node_id: String,
    private_key: PrivateKey,
    pub public_key: PublicKey,
}

impl Miner {
    pub fn new(node: &Node) -> Self {
        let mut rng = rand::thread_rng();
        let node_id = node.id.clone();
        let pvt_key = PrivateKey::generate(&mut rng);
        let pub_key = pvt_key.public_key();
        Miner { node_id, private_key: pvt_key, public_key: pub_key }
    }

    pub fn sign_message(&self, block: &Block) -> String {
        let signature = self.private_key.sign(block.serialize_block());
        hex::encode(signature.as_bytes())
    }
}

impl Node {
    pub fn new(chain: BlockChain) -> Self {
        let rng = rand::thread_rng();
        let id = rng.sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let mut node = Node { id: id, block_chain: chain, stake: 0.0, state: NodeState::Idle,
             staging: vec![], validators: vec![], primary: vec![] };
        node.propose_stake();
        node
    }

    fn serialize_node(&self) -> String{
        let json_string = to_string(&self).expect("Failed to parse Node");
        json_string
    }

    fn deserialize_node(json_str: &str) -> Self{
        let node = serde_json::from_str(json_str).expect("Failed to parse JSON");
        node
    }

    pub async fn pool_transactions(&mut self, topic: &[&str], user_base: Vec<User>, miner: &Miner) {
        
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", &self.id)
            .set("bootstrap.servers", "localhost:9092")
            .set("session.timeout.ms", "10000")
            .create()
            .expect("Consumer Creation has failed");

        println!("{:?}", self);

        consumer.subscribe(topic).expect("Failed to subscribe to topic!");

        let mut pool: Vec<Transaction> = Vec::new();

        let mut message_stream = consumer.stream();
        self.state = NodeState::PrePreparing;

        while let Some(message_result) = message_stream.next().await {
            match message_result {
                Err(e) => eprintln!("Error while receiving message: {}", e),
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        match serde_json::from_slice::<Transaction>(payload){
                            Ok(transaction) => {
                                let index = user_base.iter().position(|x|  x.user_id == transaction.from).unwrap();
                                if transaction.verify_transaction(user_base[index].public_key) == true
                                && user_base[index].balance > (transaction.amount + transaction.fee) {
                                    pool.push(transaction);
                                } else { continue; }

                                if pool.len() == 256 { 
                                    self.preprepare_phase::<Node>(pool.clone(), &miner);
                                    pool = vec![];
                                 } else { continue; }
                            }
                            Err(e) => {
                                eprintln!("Failed to deserialize message: {}", e);
                            }
                        }
                    }
                    else { println!("No payload found in message"); }
                    consumer.commit_message(&message, CommitMode::Async).expect("Can not commit message");
                }
            }
        }
    }

    pub fn concensus(){

    }
    
}

pub fn msg() -> String {
    String::from("hello")
}