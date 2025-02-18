use serde::{Serialize, Deserialize};
use crate::definitions::block_header::{Block, BlockChain};
use crate::definitions::transaction_header::Transaction;
use crate::definitions::consensus_header::Validator;
use bls_signatures::{PrivateKey, PublicKey};
use rdkafka::{consumer::StreamConsumer, producer::BaseProducer};
use std::sync::Arc;
use std::collections::HashMap;


#[derive(Serialize, Deserialize, Clone, std::fmt::Debug)]
pub enum NodeState {
    Idle,
    PrePreparing,
    Preparing,
    Committing,
    Done
}

pub struct Node {
    pub id: String,
    pub block_chain: BlockChain,
    pub stake: f64,
    pub state: NodeState,
    pub staging: Vec<Transaction>,
    pub block_staging: Vec<Block>,
    pub validators: Vec<Validator>,
    pub primary: Vec<Validator>,
    pub msg_idx: Vec<usize>,
    pub private_key: PrivateKey
}

pub struct PoolingMetrics {
    pub tps: f64,
    pub processtime: f64,
    pub bad_tx: f64,
    pub ttf: f64
}

pub struct ConcensusMetrics {
    pub prepre_time: f64,
    pub pre_time: f64,
    pub commit_time: f64,
    pub prepre_wait: f64,
    pub pre_wait: f64,
    pub commit_wait: f64
}

#[allow(async_fn_in_trait)]
pub trait NodeMethods {
    
    fn new() -> Self;

    fn sign_message(&self, block: &Block) -> String;

    async fn pool_transactions(&mut self, consumer: &StreamConsumer, user_base: &mut HashMap<String, f64>,
         residual: &mut Vec<Transaction>, time_out: u64, tx_time: u64, block_size: &usize) -> (Option<Vec<Transaction>>,Vec<Transaction>, Option<PoolingMetrics>);

    async fn concensus(&mut self, pool: Vec<Transaction>, pkey_store: HashMap<String, PublicKey>,
    prepre_con: Arc<StreamConsumer>, pre_con: Arc<StreamConsumer>,
    prepre_ready: &StreamConsumer, pre_ready: &StreamConsumer, commit_ready: &StreamConsumer,
    prepre_prod: Arc<BaseProducer>, pre_prod: Arc<BaseProducer>, comm_prod: Arc<BaseProducer>,
    time_out: u64) -> Option<ConcensusMetrics>;
}