use bls_signatures::{PublicKey, Serialize, Signature};
use log::{info,error};
use rdkafka::{consumer::{Consumer, StreamConsumer}, producer::{BaseProducer, BaseRecord, Producer}};
use rdkafka::message::Message;
use tokio::time::timeout;
use futures_util::StreamExt;
use std::time::{Duration, Instant};
use crate::definitions::network_header::{MessageTypeMethods, NodeMessage, NodeMessageMethods, MessageType, Network};

use crate::definitions::{block_header::{Block, BlockMethods}, node_header::{Node, NodeMethods}};

impl MessageTypeMethods for MessageType {
    fn unwrap(&self) -> String {
        let val: String = match &self {
            &MessageType::PrePrepare(a) => a.to_owned(),
            &MessageType::Prepare(a) => a.to_owned(),
            &MessageType::Commit(a) => a.to_owned(),
            &MessageType::Reply(a) => a.to_owned(), 
        };
        
        val
    }
}

impl NodeMessageMethods for NodeMessage {
    fn new(node: &Node, block: &Block, msg_type: String, idx: usize) -> Self {
       let sign = node.sign_message(block);
       let msg = match msg_type.to_uppercase().as_str() {
                        "PREPREPARE" => {MessageType::PrePrepare(block.serialize_block())},
                        "PREPARE" => {MessageType::Prepare(block.serialize_block())},
                        "COMMIT" => {MessageType::Commit(block.serialize_block())},
                        "REPLY" => {MessageType::Reply(block.serialize_block())},
                        _ => panic!("Invalid message type")
                };
        NodeMessage { msg_type: msg, block: block.clone(), signature: sign, sender_id: node.id.clone(), seq_num: idx }
    }

    fn deserialize_message(msg_json: String) -> NodeMessage {
        let node_msg: NodeMessage = serde_json::from_str(&msg_json).expect("Failed to deserialize the message");
        node_msg
    }

    fn verify_message(self, pub_key: PublicKey) -> bool {
        pub_key.verify(Signature::from_bytes(hex::decode(&self.signature).unwrap().as_slice())
            .expect("Failed to parse the signature"),
             self.block.serialize_block())
    }
}

impl Network for Node {
    async fn broadcast_kafka(&self, topic: &str, message: NodeMessage, producer: &BaseProducer){
        info!("Brodcasting message to topic: {}", topic);

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

        producer.flush(Duration::from_secs(2)).expect("Failed to flush");
        info!("Message broadcasted");
    }

    async fn consume_kafka(id: String, hash: String, topic: &str, consumer: &StreamConsumer, producer: &BaseProducer,time_out: u64) -> Option<Vec<String>>{

        info!("Listening to topic: {}", topic);

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
        
                producer.flush(Duration::from_secs(2)).expect("Failed to flush");

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
            None
        }
        else {
            Some(message_pool)
        }
    }

    // Listen for status of peer nodes before proceeding
    async fn ready_state(thresh: usize, topic: String, consumer: &StreamConsumer, time_out: u64) -> (bool, f64) {

        info!("Listening for readiness on topic: {}", topic);

        let mut msg_stream = consumer.stream();

        let mut message_pool = Vec::new();

        let mut retries = 0;

        let start = Instant::now();

        loop {
            if message_pool.len() == thresh {
                let end_time = start.elapsed().as_millis() as f64;
                info!("All nodes are ready");
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
                    if retries > 5 {break;} else {continue;}
                } else {
                    continue;
                }
            }
            }
        }

        error!("Not all nodes are ready, synchronization failed");
        (false, 0.0)
    }
}