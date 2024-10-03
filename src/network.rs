use rdkafka::{consumer::{BaseConsumer, Consumer}, producer::{BaseProducer, BaseRecord, Producer}, ClientConfig};
use rdkafka::message::Message;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

use crate::{block::Block, node::{Miner, Node}};

#[derive(Serialize, Deserialize, Clone)]
pub enum MessageType {
    Request(String),
    PrePrepare(String),
    Prepare(String),
    Commit(String),
    Reply(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeMessage {
    pub msg_type: MessageType,
    pub sender_id: String,
    pub seq_num: u64
}

pub trait Network {
    fn broadcast_kafka(&self, topic: &str, message: NodeMessage);
    
    fn consume_kafka(topic: &str) -> Vec<String>;
}

impl NodeMessage {
    pub fn new(miner: Miner, block: Block, msg_type: String) -> Self {
       let hash = miner.sign_message(&block);
       let msg = match msg_type.to_uppercase().as_str() {
                        "REQUEST" => { MessageType::Request(hash) },
                        "PREPREPARE" => {MessageType::PrePrepare(hash)},
                        "PREPARE" => {MessageType::Prepare(hash)},
                        "COMMIT" => {MessageType::Commit(hash)},
                        "REPLY" => {MessageType::Reply(hash)},
                        _ => panic!("Invalid message type")
                };
        NodeMessage { msg_type: msg, sender_id: miner.node_id, seq_num: 0 }
    }

    pub fn deserialize_message(msg_json: String) -> NodeMessage {
        let node_msg: NodeMessage = serde_json::from_str(&msg_json).expect("Failed to deserialize the message");
        node_msg
    }
}

impl Network for Node {
    fn broadcast_kafka(&self, topic: &str, message: NodeMessage){
        let message = serde_json::to_string(&message).expect("Failed to parse Block");
        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .create()
            .expect("Failed to create producer");

        producer.send(
            BaseRecord::to(topic)
            .payload(&message)
            .key(&self.id)
        ).expect("Failed to send message");

        producer.flush(Duration::from_secs(1)).expect("Failed to flush producer");
    }

    fn consume_kafka(topic: &str) -> Vec<String>{
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.server", "localhost:9092")
            .create()
            .expect("Failed to make consumer");

        consumer.subscribe(&[topic]).expect("Subscription Error");

        let timeout = Duration::from_secs(10);
        let mut message_pool = Vec::new();
        let mut last_received_time = None;

        println!("Listening for messages in topic: {}", topic);

        loop {
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
                None => { continue; }
            }
        }
        message_pool
    }
}