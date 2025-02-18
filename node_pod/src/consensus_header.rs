use serde::{Serialize, Deserialize};
use crate::node_header::Node;
use rdkafka::producer::BaseProducer;
use std::future::Future;
use crate::transaction_header::Transaction;
use std::collections::HashMap;
use bls_signatures::PublicKey;

#[derive(Serialize, Deserialize, Clone)]
pub struct Stake {
    pub node_id: String,
    pub stake: f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Validator {
    pub node_id: String,
    pub public_key: String
}

pub trait StakeMethods {
    fn new(node: &Node, stake: f64) -> Self;

    fn serialize(&self) -> String;

    fn deserialize(json_str: String) -> Self;
}

pub trait PoS {

    fn propose_stake(&mut self, producer: &BaseProducer) -> impl Future<Output = ()> + Send;

}

pub trait Pbft {

    fn preprepare_phase(&mut self, _pool: Vec<Transaction>,_producer: &BaseProducer) -> impl Future<Output = ()> + Send;

    fn prepare_phase(&mut self, _pkey_store: &HashMap<String, PublicKey>, _primary_msg: Vec<String>,_producer: &BaseProducer) -> impl Future<Output = ()> + Send;

    fn commit_phase(&mut self, _pkey_store: &HashMap<String, PublicKey>, _prepare_msg: Vec<String>,_producer: &BaseProducer) -> impl Future<Output = bool> + Send;
    
}

pub trait ValidatorMethods {

    fn serialize(&self) -> String;

    fn deserialize(json_str: String) -> Self;

}