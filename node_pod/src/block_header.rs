use serde::{Serialize, Deserialize};
use crate::transaction_header::Transaction;

#[derive(Serialize, Deserialize, Clone, std::fmt::Debug, PartialEq)]
pub struct Block {
    pub index: u64,
    pub hash: String,
    pub timestamp: u64,
    pub merkle_root: String,
    pub prev_hash: String,
    pub transactions: Vec<Transaction>
}

#[derive(Serialize, Deserialize, Clone, std::fmt::Debug)]
pub struct BlockChain {
    pub chain: Vec<Block>,
}

pub trait BlockMethods {

    fn new(data: Vec<Transaction>, previous_hash: String, idx: u64) -> Self;

    fn new_genesis (data: Vec<Transaction>, previous_hash: String, idx: u64) -> Self;

    fn hash(&self) -> Vec<u8>;

    fn serialize_block(&self) -> String;

    fn deserialize_block(json_string: &str) -> Self;

    fn validate(&self, transactions: Vec<Transaction>) -> bool;

    fn is_equal(&self, block: Block) -> bool;

}

pub trait BlockChainMethods {

    fn new() -> Self;

    fn add_block(&mut self, block: Block);

    fn validate_transaction(&self, data: Vec<Transaction>, block_index: usize) -> bool;

    fn verify_chain(&self) -> bool;

    fn serialize(&self) -> String;

    fn deserialize(json: String) -> Self;

}