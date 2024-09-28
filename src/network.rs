use rdkafka::{consumer::{BaseConsumer, CommitMode, Consumer, StreamConsumer}, error::KafkaResult, message, producer::{BaseProducer, BaseRecord, FutureProducer, FutureRecord, Producer}, ClientConfig};
use rdkafka::message::Message;
use std::time::Duration;
use futures_util::stream::StreamExt;
use serde_json::{to_string, from_str};
use serde::{Deserialize, Serialize};

use crate::node::Node;

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
    
    fn consume_kafka(topic: &str, group_id: &str) -> Vec<String>;
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

    fn consume_kafka(topic: &str, group_id: &str) -> Vec<String>{
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.server", "localhost:9092")
            .set("group.id", group_id)
            .create()
            .expect("Failed to make consumer");

        consumer.subscribe(&[topic]).expect("Subscription Error");

        let mut message_pool = Vec::new();

        println!("Listening for messages in topic: {}", topic);

        loop {
            match consumer.poll(Duration::from_secs(1)) {
                Some(Ok(message)) => {
                    if let Some(payload) = message.payload() {
                        let msg = String::from(std::str::from_utf8(payload).expect("Failed to deserialize message"));
                        message_pool.push(msg);
                    }
                }
                _ => continue,
            }
        }
        message_pool
    }
}