use bls_signatures::{PublicKey, Serialize, Signature};
use rdkafka::{consumer::{BaseConsumer, Consumer, StreamConsumer}, producer::{BaseProducer, BaseRecord}, ClientConfig};
use rdkafka::message::Message;
use rdkafka::producer::Producer;
use tokio::time::timeout;
use futures_util::StreamExt;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize as SerdeSerialize};

use crate::{block::Block, node::Node};

#[derive(SerdeSerialize, Deserialize, Clone)]
pub enum MessageType {
    PrePrepare(String),
    Prepare(String),
    Commit(String),
    Reply(String),
}

impl MessageType {
    pub fn unwrap(&self) -> String {
        let val: String = match &self {
            &MessageType::PrePrepare(a) => a.to_owned(),
            &MessageType::Prepare(a) => a.to_owned(),
            &MessageType::Commit(a) => a.to_owned(),
            &MessageType::Reply(a) => a.to_owned(), 
        };
        
        return val;
    }
}

// pub struct PrepareMessage {
//     block: Block,
//     node_id: String,
//     signature: String,
//     seq_num: i32
// }

// impl PrepareMessage {
//     pub fn new(block: Block, node_id: String, signature: String, seq_num: i32) -> Self {
//         PrepareMessage { block, node_id , signature, seq_num }
//     }
// }

#[derive(SerdeSerialize, Deserialize, Clone)]
pub struct NodeMessage {
    pub msg_type: MessageType,
    pub block: Block,
    pub signature: String,
    pub sender_id: String,
    pub seq_num: usize
}

pub trait Network {
    fn broadcast_kafka(&self, topic: &str, message: NodeMessage, producer: &BaseProducer) -> impl std::future::Future<Output = ()> + Send;
    
    fn consume_kafka(&self, topic: &str) -> Vec<String>;
}

impl NodeMessage {
    pub fn new(node: &Node, block: &Block, msg_type: String, idx: usize) -> Self {
       let sign = node.sign_message(&block);
       let msg = match msg_type.to_uppercase().as_str() {
                        "PREPREPARE" => {MessageType::PrePrepare(block.serialize_block())},
                        "PREPARE" => {MessageType::Prepare(block.serialize_block())},
                        "COMMIT" => {MessageType::Commit(block.serialize_block())},
                        "REPLY" => {MessageType::Reply(block.serialize_block())},
                        _ => panic!("Invalid message type")
                };
        NodeMessage { msg_type: msg, block: block.clone(), signature: sign, sender_id: node.id.clone(), seq_num: idx }
    }

    pub fn deserialize_message(msg_json: String) -> NodeMessage {
        let node_msg: NodeMessage = serde_json::from_str(&msg_json).expect("Failed to deserialize the message");
        node_msg
    }

    pub fn verify_message(self, pub_key: PublicKey) -> bool {
        pub_key.verify(Signature::from_bytes(hex::decode(&self.signature).unwrap().as_slice())
            .expect("Failed to parse the signature"),
             &self.block.serialize_block())
    }
}

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

pub async fn consume_kafka(id: String, hash: String, topic: &str, consumer: &StreamConsumer, producer: &BaseProducer,time_out: u64) -> Option<Vec<String>>{

    let prod_id = id + &hash + topic;

    let mut first = true;

    let mut msg_stream = consumer.stream();

    let mut message_pool = Vec::new();

    let mut retries = 0;

    loop {
        if first {
            // broadcast a message that we are listening the topic T
            producer.send(
                BaseRecord::to("Status")
                .payload(topic)
                .key(&prod_id)
            ).expect("Failed to send message");
    
            producer.flush(Duration::from_secs(20)).expect("Failed to flush");

            first = false;
        }
        match timeout(Duration::from_millis(time_out), msg_stream.next()).await {
        Ok(Some(message_result)) => {
            match message_result {
                Err(e) => eprintln!("Error while receiving message: {}", e),
                Ok(message) => {
                        if let Some(payload) = message.payload() {
                            let msg = String::from(std::str::from_utf8(payload).expect("Failed to deserialize message"));
                            message_pool.push(msg);
                        }
                        if let Err(e) = consumer.commit_message(&message, rdkafka::consumer::CommitMode::Sync) {
                            eprintln!("Failed to commit message: {}", e);
                        }
                    }
                }
            }
        Ok(_) => {
            println!("Stream Ended..");
            msg_stream = consumer.stream();
        }
        Err(_) => {
            if !message_pool.is_empty(){
                retries += 1;
                if retries > 15 {break;} else {continue;}
            } else {
                continue;
            }
        }
        }
    }
    if message_pool.is_empty() {
        return None;
    }
    else {
        return Some(message_pool);
    }
}

pub async fn ready_state(thresh: usize, topic: String, consumer: &StreamConsumer, time_out: u64) -> (bool, f64){

    let mut msg_stream = consumer.stream();

    let mut message_pool = Vec::new();

    let mut retries = 0;

    let start = Instant::now();

    loop {
        if message_pool.len() == thresh {
            let end_time = start.elapsed().as_millis() as f64;
            return (true, end_time);
        }

        match timeout(Duration::from_millis(time_out), msg_stream.next()).await {
        Ok(Some(message_result)) => {
            match message_result {
                Err(e) => eprintln!("Error while receiving message: {}", e),
                Ok(message) => {
                        if let Some(payload) = message.payload() {
                            let msg = String::from(std::str::from_utf8(payload).expect("Failed to deserialize message"));
                            if msg == topic {
                                message_pool.push(msg);
                            }
                            else { continue; }
                        }
                    }
                }
            }
        Ok(_) => {
            println!("Stream Ended..");
            msg_stream = consumer.stream();
        }
        Err(_) => {
            if !message_pool.is_empty(){
                retries += 1;
                if retries > 15 {break;} else {continue;}
            } else {
                continue;
            }
        }
        }
    }
    return (false, 0.0);
}