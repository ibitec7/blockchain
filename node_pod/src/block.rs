use std::{time::UNIX_EPOCH, vec};
use openssl::sha;
use hex;
use serde_json::{to_string, from_str};

use crate::transaction_header::Transaction;
use crate::merkle_header::{MerkleTree, MerkleMethods};
use crate::block_header::{Block, BlockMethods, BlockChain, BlockChainMethods};

impl BlockMethods for Block {
    fn new(data: Vec<Transaction>, previous_hash: String, idx: u64) -> Self {
        let root = hex::encode(MerkleTree::generate_root(&data));

        let mut ts: u64 = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        ts -= ts % 5;

        let mut block = Block {
            index: idx,
            hash: String::new(),
            timestamp: ts,
            merkle_root: root,
            prev_hash: previous_hash,
            transactions: data
        };

        let block_hash = hex::encode(block.hash());

        block.hash = block_hash;

        block
    }

    fn new_genesis (data: Vec<Transaction>, previous_hash: String, idx: u64) -> Self {
        let root = String::from("");

        let mut ts = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        ts -= ts % 60;

        let mut block = Block {
            index: idx,
            hash: String::new(),
            timestamp: ts,
            merkle_root: root,
            prev_hash: previous_hash,
            transactions: data
        };

        let block_hash = hex::encode(block.hash());

        block.hash = block_hash;

        block
    }

    fn hash(&self) -> Vec<u8> {
        let mut hasher = sha::Sha256::new();
        hasher.update(&self.index.to_be_bytes());
        hasher.update(&self.timestamp.to_be_bytes());
        hasher.update(&hex::decode(&self.merkle_root).expect("Failed to decode merkle_root hash"));
        hasher.update(&hex::decode(&self.prev_hash).expect("Failed to decode previous Block Hash"));

        return hasher.finish().to_vec();
    }

    fn serialize_block(&self) -> String {
        let json_string = to_string(&self).expect("Failed to serialize block");
        json_string
    }

    fn deserialize_block(json_string: &str) -> Self {
        let block: Block = from_str(json_string).expect("Failed to parse block");
        block
    }

    fn validate(&self, transactions: Vec<Transaction>) -> bool {
        let new_root = MerkleTree::generate_root(&transactions);
        let new_root_str = hex::encode(&new_root);

        let correct_root = new_root_str == self.merkle_root;

        if !correct_root { println!["Merkle roots do not match"] }

        correct_root
    }

    fn is_equal(&self, block: Block) -> bool {
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

impl BlockChainMethods for BlockChain {
    fn new() -> Self {
        let genesis_block = Block::new_genesis(vec![], hex::encode(vec![]), 0);

        return BlockChain { chain: vec![genesis_block] };
    }

    fn add_block(&mut self, block: Block) {        
        self.chain.push(block);
    }

    fn validate_transaction(&self, data: Vec<Transaction>, block_index: usize) -> bool {
        self.chain[block_index].validate(data)
    }

    fn verify_chain(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let prev_block = &self.chain[i - 1];
            if hex::decode(&current_block.prev_hash).expect("Failed to decode previous hash") != prev_block.hash(){
                return false;
            }
        }
        true
    }

    fn serialize(&self) -> String {
        let json_string = to_string(&self).expect("Failed to serialize Blockchain");
        json_string
    }

    fn deserialize(json: String) -> Self {
        let chain: BlockChain = from_str(&json).expect("Failed to parse JSON");
        chain
    }
}