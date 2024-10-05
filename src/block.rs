use std::{collections::HashMap, time::UNIX_EPOCH, vec};
use bls_signatures::PublicKey;
use openssl::sha;
use hex;
use serde::{ Serialize, Deserialize };
use serde_json::{to_string, from_str};
use crate::transaction::Transaction;
use crate::merkle_tree;

#[derive(Serialize, Deserialize, Clone, std::fmt::Debug)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub merkle_root: String,
    pub nonce: u64,
    pub prev_hash: String
}

impl Block {
    pub fn new(data: Vec<Transaction>, hash: String, idx: u64) -> Self {
        let root = hex::encode(merkle_tree::generate_root(data));

        Block {
            index: idx,
            timestamp: std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            merkle_root: root,
            nonce: 0,
            prev_hash: hash
        }

    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = sha::Sha256::new();
        hasher.update(&self.index.to_be_bytes());
        hasher.update(&self.timestamp.to_be_bytes());
        hasher.update(&self.nonce.to_be_bytes());
        hasher.update(&hex::decode(&self.merkle_root).expect("Failed to decode merkle_root hash"));
        hasher.update(&hex::decode(&self.prev_hash).expect("Failed to decode previous Block Hash"));

        return hasher.finish().to_vec();
    }

    pub fn serialize_block(&self) -> String {
        let json_string = to_string(&self).expect("Failed to serialize block");
        json_string
    }

    pub fn deserialize_block(json_string: &str) -> Self {
        let block: Block = from_str(json_string).expect("Failed to parse block");
        block
    }

    pub fn validate(&self, transactions: Vec<Transaction>) -> bool {
        let new_root = merkle_tree::generate_root(transactions);
        new_root == hex::decode(&self.merkle_root).expect("Failed to decode Block merkle_root")
    }
}

#[derive(Serialize, Deserialize)]
pub struct Validator{
    pub address: String,
    pub stake: u64,
}

#[derive(Serialize, Deserialize, Clone, std::fmt::Debug)]
pub struct BlockChain {
    pub chain: Vec<Block>,
}

impl BlockChain {
    pub fn new() -> Self {
        let genesis_block = Block {
            index: 0,
            timestamp: 0,
            merkle_root: hex::encode(vec![0;32]),
            nonce: 0,
            prev_hash: hex::encode(vec![]),
        };

        return BlockChain { chain: vec![genesis_block] };
    }

    pub fn add_block(&mut self, data: Vec<Transaction>) {
        let prev_block = &self.chain[self.chain.len() - 1];

        let new_block = Block::new(data, hex::encode(prev_block.hash()), prev_block.index + 1);
        
        self.chain.push(new_block);
    }

    pub fn validate_transaction(&self, data: Vec<Transaction>, block_index: usize) -> bool {
        self.chain[block_index].validate(data)
    }

    pub fn verify_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let prev_block = &self.chain[i - 1];
            if hex::decode(&current_block.prev_hash).expect("Failed to decode previous hash") != prev_block.hash(){
                return false;
            }
        }
        true
    }

    pub fn serialize(&self) -> String {
        let json_string = to_string(&self).expect("Failed to serialize Blockchain");
        json_string
    }

    pub fn deserialize(json: String) -> Self {
        let chain: BlockChain = from_str(&json).expect("Failed to parse JSON");
        chain
    }
}