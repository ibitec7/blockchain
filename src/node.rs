use std::{ops::{Deref, DerefMut}, time::Duration};

use crate::{block::{Block, BlockChain}, concensus::{self, PoS}, network::{self, MessageType, Network, NodeMessage}, simulate::User, transaction::Transaction};
use rand::{distributions::Alphanumeric, Rng};

use rdkafka::{consumer::{BaseConsumer, CommitMode, Consumer, StreamConsumer}, error::KafkaResult, message, producer::{BaseProducer, BaseRecord, FutureProducer, FutureRecord, Producer}, ClientConfig};
use rdkafka::message::Message;
use futures_util::stream::StreamExt;
use serde_json::{to_string, from_str};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum NodeState {
    Idle,
    PrePreparing,
    Preparing,
    Committing,
    Done
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Node {
    pub id: String,
    pub block_chain: BlockChain,
    pub stake: f64,
    pub state: NodeState,
    pub staging: Vec<Transaction>,
    pub validators: Vec<String>,
    pub primary: Vec<String>,
}

impl Node {
    pub fn new(chain: BlockChain) -> Self {
        let rng = rand::thread_rng();
        let id = rng.sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let mut node = Node { id: id, block_chain: chain, stake: 0.0, state: NodeState::Idle, staging: vec![], validators: vec![], primary: vec![] };
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

    pub async fn pool_transactions(&mut self, topic: &[&str], user_base: Vec<User>) -> KafkaResult<Vec<Transaction>> {

        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", &self.id)
            .set("bootstrap.servers", "localhost:9092")
            .set("session.timeout.ms", "10000")
            .create()
            .expect("Consumer Creation has failed");

        consumer.subscribe(topic).expect("Failed to subscribe to topic!");

        let mut pool: Vec<Transaction> = Vec::new();

        let mut message_stream = consumer.stream();
        let mut stop_flag = false;
        self.state = NodeState::PrePreparing;

        while let Some(message_result) = message_stream.next().await {
            match message_result {
                Err(e) => eprintln!("Error while receiving message: {}", e),
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        match serde_json::from_slice::<Transaction>(payload){
                            Ok(transaction) => {
                                println!(
                                    "Received Transaction: {:?}     |       From Partition: {}",
                                    transaction.serialize(),
                                    message.partition()
                                );
                                let index = user_base.iter().position(|x|  x.user_id == transaction.from).unwrap();
                                if transaction.verify_transaction(user_base[index].public_key) == true
                                && user_base[index].balance > (transaction.amount + transaction.fee) {
                                    pool.push(transaction);
                                } else { continue; }
                                if self.staging.len() == 256 { println!("{}",self.staging.len()); stop_flag = true; } else { continue; }
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

            if stop_flag || pool.len() > 256 {
                break;
            }
        }

        self.staging = pool.clone();
        println!("prev_hash: {:?}", hex::encode(
            self.block_chain.chain[self.block_chain.chain.len() - 1].hash()));
        println!("index: {:?}", self.block_chain.chain[self.block_chain.chain.len() - 1].index + 1);
        let block = Block::new(pool.clone(), 
            hex::encode(
            self.block_chain.chain[self.block_chain.chain.len() - 1].hash()), 
            self.block_chain.chain[self.block_chain.chain.len() - 1].index + 1);

        let message = NodeMessage { msg_type: MessageType::Request(block.serialize_block()), sender_id: self.id.clone(), seq_num: 1 };
        let primary = self.primary.clone();
        let id = self.id.clone();
        let mut is_primary = false;
        for leader in primary {
            if leader == id {
                is_primary = true
            } else { continue; }
        }
        if is_primary {
            self.broadcast_kafka("PrePrepare", message);
        }
        Ok(pool)
    }

    pub fn concensus(){

    }
    
}

pub fn msg() -> String {
    String::from("hello")
}