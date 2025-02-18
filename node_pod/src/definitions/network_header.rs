use serde::{Serialize, Deserialize};
use crate::definitions::block_header::Block;
use crate::definitions::node_header::Node;
use bls_signatures::PublicKey;
use rdkafka::{producer::BaseProducer, consumer::StreamConsumer};

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

#[allow(async_fn_in_trait)]
pub trait NodeMessageMethods {

    fn new(node: &Node, block: &Block, msg_type: String, idx: usize) -> Self;

    fn deserialize_message(msg_json: String) -> NodeMessage;

    fn verify_message(self, pub_key: PublicKey) -> bool;


}

#[allow(async_fn_in_trait)]
pub trait Network {
    fn broadcast_kafka(&self, topic: &str, message: NodeMessage, producer: &BaseProducer) -> impl std::future::Future<Output = ()> + Send;
    
    async fn consume_kafka(id: String, hash: String, topic: &str, consumer: &StreamConsumer, producer: &BaseProducer,time_out: u64) -> Option<Vec<String>>;

    async fn ready_state(thresh: usize, topic: String, consumer: &StreamConsumer, time_out: u64) -> (bool, f64);

}