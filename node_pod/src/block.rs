use std::{time::UNIX_EPOCH, vec};
use openssl::sha;
use hex;
use serde::{ Serialize, Deserialize };
use serde_json::{to_string, from_str};

use crate::transaction::Transaction;
use crate::merkle_tree;

#[derive(Serialize, Deserialize, Clone, std::fmt::Debug, PartialEq)]
pub struct Block {
    pub index: u64,
    pub hash: String,
    pub timestamp: u64,
    pub merkle_root: String,
    pub prev_hash: String,
    pub transactions: Vec<Transaction>
}

impl Block {
    pub fn new(data: Vec<Transaction>, previous_hash: String, idx: u64) -> Self {
        let root = hex::encode(merkle_tree::generate_root(data.clone()));

        let mut ts: u64 = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        ts -= ts % 5;

        let temp_block = Block {
            index: idx,
            hash: String::new(),
            timestamp: ts,
            merkle_root: root.clone(),
            prev_hash: previous_hash.clone(),
            transactions: data.clone()
        };

        let block_hash = hex::encode(temp_block.hash());

        Block {
            index: idx,
            hash: block_hash,
            timestamp: std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            merkle_root: root,
            prev_hash: previous_hash,
            transactions: data
        }
    }

    pub fn new_genesis (data: Vec<Transaction>, previous_hash: String, idx: u64) -> Self {
        let root = String::from("");

        let mut ts = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        ts -= ts % 60;

        let temp_block = Block {
            index: idx,
            hash: String::new(),
            timestamp: ts,
            merkle_root: root.clone(),
            prev_hash: previous_hash.clone(),
            transactions: data.clone()
        };

        let block_hash = hex::encode(temp_block.hash());

        Block {
            index: idx,
            hash: block_hash,
            timestamp: ts,
            merkle_root: root,
            prev_hash: previous_hash,
            transactions: data
        }
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = sha::Sha256::new();
        hasher.update(&self.index.to_be_bytes());
        hasher.update(&self.timestamp.to_be_bytes());
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
        let new_root_str = hex::encode(&new_root);

        let correct_root = new_root_str == self.merkle_root;

        if !correct_root { println!["Merkle roots do not match"] }

        correct_root
    }

    pub fn is_equal(&self, block: Block) -> bool {
        let mut predicate: bool = self.index == block.index;
        if !predicate { panic!["index not the same"] }
        predicate = predicate && (self.timestamp == block.timestamp);
        if !predicate { panic!["timestamp not the same"] };
        predicate = predicate && (self.merkle_root == block.merkle_root);
        if !predicate { panic![ "merkle_root not the same" ]}
        predicate = predicate && (self.prev_hash == block.prev_hash);
        if !predicate { panic!["prev_hash not the same"] }
        predicate = predicate && (self.transactions == block.transactions);
        if !predicate { panic!["transactions not the same"] };
        predicate
    }
}

#[derive(Serialize, Deserialize, Clone, std::fmt::Debug)]
pub struct BlockChain {
    pub chain: Vec<Block>,
}

impl BlockChain {
    pub fn new() -> Self {
        let genesis_block = Block::new_genesis(vec![], hex::encode(vec![]), 0);

        return BlockChain { chain: vec![genesis_block] };
    }

    pub fn add_block(&mut self, block: Block) {        
        self.chain.push(block);
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