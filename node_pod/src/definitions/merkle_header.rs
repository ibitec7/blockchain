use serde::{Serialize, Deserialize};
use crate::definitions::transaction_header::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    pub merkle_root: Vec<u8>,
    pub leaves: Vec<Vec<u8>>,
    pub nodes: Vec<Vec<u8>>,
    pub depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub path: Vec<Vec<u8>>,
    pub leaf_index: usize,
}

pub trait MerkleMethods {

    fn new_genesis() -> Self;

    fn new(data: &[Transaction]) -> Self;

    fn generate_proof(&self, target: &Transaction) -> Proof;

    fn generate_root(data: &[Transaction]) -> Vec<u8>;

    fn validate_proof(proof: &Proof, leaf: Transaction, merkle_root: &[u8]) -> bool;

}
