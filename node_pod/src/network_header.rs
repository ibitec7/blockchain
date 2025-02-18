use serde::{Serialize, Deserialize};
use crate::block_header::Block;
use crate::node_header::Node;
use bls_signatures::PublicKey;
use rdkafka::producer::BaseProducer;

#[derive(Serialize, Deserialize, Clone)]
pub enum MessageType {
    PrePrepare(String),
    Prepare(String),
    Commit(String),
    Reply(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeMessage {
    pub msg_type: MessageType,
    pub block: Block,
    pub signature: String,
    pub sender_id: String,
    pub seq_num: usize
}

pub trait MessageTypeMethods {
    
    fn unwrap(&self) -> String;

}

pub trait NodeMessageMethods {

    fn new(node: &Node, block: &Block, msg_type: String, idx: usize) -> Self;

    fn deserialize_message(msg_json: String) -> NodeMessage;

    fn verify_message(self, pub_key: PublicKey) -> bool;

}

pub trait Network {
    fn broadcast_kafka(&self, topic: &str, message: NodeMessage, producer: &BaseProducer) -> impl std::future::Future<Output = ()> + Send;
    
    fn consume_kafka(&self, topic: &str) -> Vec<String>;
}