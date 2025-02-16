use openssl::sha::Sha256;
use std::ops::Deref;
use crate::merkle_header::{MerkleTree, Proof, MerkleMethods};
use crate::transaction_header::Transaction;

impl MerkleMethods for MerkleTree {

    fn new_genesis() -> Self {
        MerkleTree {
            merkle_root: Vec::new(),
            leaves: Vec::new(),
            nodes: Vec::new(),
            depth: 0
        }
    }

    fn new(data: &Vec<Transaction>) -> Self {
        let transactions: Vec<Vec<u8>> = data.iter().map(|x| -> Vec<u8> { hex::decode(x.id.clone()).unwrap() }).collect();
        let leaves = transactions.clone();

        let mut nodes = Vec::new();

        let mut current_level = leaves.clone();
        let mut next_level = Vec::new();
        let mut levels = 1;
        let current_level_it = current_level.clone();

        for leaf in current_level_it {
            nodes.push(leaf);
        }
        
        while current_level.len() > 1 {
            next_level.clear();
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    left
                };
                let mut combined = left.clone();
                let mut other = right.clone();
                combined.append(&mut other);
                let mut hasher = Sha256::new();
                hasher.update(&combined);
                let hash = hasher.finish();
                let hash_clone = hash.clone();


                next_level.push(hash.to_vec());
                nodes.push(hash_clone.to_vec());
            }

            current_level = next_level.clone();
            levels += 1;
        }

        let merkle_root = current_level.first().cloned().expect("Failed to extract Merkle Root");
        MerkleTree { merkle_root, leaves, nodes, depth: levels }
    }

    fn generate_proof(&self, target: &Transaction) -> Proof {
        let mut path: Vec<Vec<u8>> = Vec::new();
        let mut current_index = self.leaves.iter().position(|x| { x.deref() == hex::decode(target.id.clone()).unwrap() })
                                        .expect("Failed to retrieve current_index");
        let mut level_size = self.leaves.len();
        let leaf_index = current_index.clone();

        match current_index % 2 {
            0 => { path.push(self.nodes[current_index].clone()); path.push(self.nodes[current_index + 1].clone()) },
            _ => { path.push(self.nodes[current_index - 1].clone()); path.push(self.nodes[current_index].clone()) }
        };

        while level_size > 2 {
            let sibling_index = if current_index % 2 == 0 { current_index + 1 } else { current_index - 1 };
            let hash :Vec<u8>;
            match current_index % 2 {
                0 => {
                    let mut hasher = Sha256::new();
                    hasher.update([self.nodes[current_index].clone(), self.nodes[sibling_index].clone()].concat().as_slice());
                    hash = hasher.finish().to_vec();
                }
                _ => {
                    let mut hasher = Sha256::new();
                    hasher.update([self.nodes[sibling_index].clone(), self.nodes[current_index].clone()].concat().as_slice());
                    hash = hasher.finish().to_vec();
                }
            };
            current_index += level_size;
            match current_index % 2 {
                0 => { path.push(hash); path.push(self.nodes[current_index + 1].clone()); },
                _ => { path.push(self.nodes[current_index - 1].clone()); path.push(hash); }
            }
            level_size /= 2;
        }

        Proof { path, leaf_index }
    }

    fn generate_root(data: &Vec<Transaction>) -> Vec<u8> {
        let transactions: Vec<Vec<u8>> = data.iter().map(|x| -> Vec<u8> {hex::decode(x.id.clone()).unwrap()}).collect();

        let mut current_level = transactions.clone();
        let mut next_level = Vec::new();

        while current_level.len() > 1 {
            next_level.clear();
            for i in (0..current_level.len()).step_by(2) {
                let current_node = &current_level[i];
                let sibling_node = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    current_node
                };
                let mut combined = current_node.clone();
                combined.append(sibling_node.clone().as_mut());
                let mut hasher = Sha256::new();
                hasher.update(&combined);
                let hash = hasher.finish().to_vec();

                next_level.push(hash)
            }
            current_level = next_level.clone();
        }
        current_level.first().expect("Failed to extract the Merkle Root").deref().to_vec()
    }

    fn validate_proof(proof: &Proof, leaf: Transaction, merkle_root: &Vec<u8>) -> bool {
        let mut sibling;
        let mut hash = hex::decode(leaf.id).unwrap();

        for _ in 0..proof.path.len() / 2 {
            let idx = proof.path.iter().position(|x| *x == hash);
            let idx = match idx {
                Some(val) => val,
                None => { return false; }
            };

            let mut hasher = Sha256::new();
            
            if idx % 2 == 0 {
                sibling = proof.path[&idx + 1].clone();

                hasher.update(&hash);
                hasher.update(&sibling);
            } else {
                sibling = proof.path[&idx - 1].clone();

                hasher.update(&sibling);
                hasher.update(&hash);
            }

            hash = hasher.finish().to_vec();
        }

        hash == *merkle_root
    }
}



