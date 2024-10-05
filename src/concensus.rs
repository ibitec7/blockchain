use std::any::{Any, TypeId};

use bls_signatures::{PublicKey, Serialize};
use std::collections::HashMap;
use rand::Rng;
use crate::node::Miner;
use crate::{block::Block, network::Network, node::Node, transaction::Transaction};
use crate::network::{NodeMessage, MessageType};

pub trait PoS {
    fn propose_stake(&mut self);

    fn set_validators(&mut self, validators: Vec<String>, primary: Vec<String>);
}

pub trait Pbft {
    fn preprepare_phase<T: Pbft + Network> (&mut self, pool: Vec<Transaction>, miner: &Miner){}

    fn prepare_phase(&self, pkey_store: HashMap<String, PublicKey>){}

    fn commit_phase(){}

    fn reply_phase(){}
}

impl PoS for Node {
    fn propose_stake(&mut self){
        let dist = rand::distributions::Uniform::new(10.0, 500.0);
        let mut rng = rand::thread_rng();
        self.stake = rng.sample(dist);
    }

    fn set_validators(&mut self, validators: Vec<String>, primary: Vec<String>) {
        self.validators = validators;
        self.primary = primary;
    }
}

impl Pbft for Node {
    fn preprepare_phase<T: Pbft + Network> (&mut self, pool: Vec<Transaction>, miner: &Miner) {
        self.staging = pool.clone();
        println!("prev_hash: {:?}", hex::encode(
            self.block_chain.chain[self.block_chain.chain.len() - 1].hash()));
        println!("index: {:?}", self.block_chain.chain[self.block_chain.chain.len() - 1].index + 1);
        let block = Block::new(pool, 
            hex::encode(
            self.block_chain.chain[self.block_chain.chain.len() - 1].hash()), 
            self.block_chain.chain[self.block_chain.chain.len() - 1].index + 1);

        let message = NodeMessage::new(miner, block, String::from("Preprepare"), self.msg_idx[0]);

        let primary = self.primary.clone();
        let id = self.id.clone();
        let mut is_primary = false;
        for leader in primary {
            if leader == id {
                is_primary = true
            } else { continue; }
        }
        if is_primary {
            self.broadcast_kafka("PrePrepare", message);
        }
    }

    fn prepare_phase(&self, pkey_store: HashMap<String, PublicKey>) {
        let primary_msg = Node::consume_kafka("Preprepare");
        let mut messages: Vec<NodeMessage> = vec![];
        
        for msg in primary_msg {
            messages.push(NodeMessage::deserialize_message(msg));
        }

        for msg in messages {
            let is_leader: bool = self.primary.contains(&msg.sender_id);
            let is_preprepare = match msg.msg_type {
                MessageType::PrePrepare(_) => true,
                _ => false
            };
            let pkey = pkey_store.get(&msg.sender_id).expect("Sender is not in Key Store");
            let verify = pkey.verify(bls_signatures::Signature::from_bytes(
                &hex::decode(msg.signature).unwrap()).unwrap()
                , hex::decode(msg.msg_type.unwrap()).unwrap());
            let is_current = self.block_chain.chain.last().unwrap().index == msg.seq_num;

            if is_leader && is_preprepare && is_current && verify {
                let new_block = Block::deserialize_block(&msg.msg_type.unwrap());
                
            }
        }

    }
}

pub fn select_validators(nodes: Vec<Node>) -> (Vec<String>, Vec<String>){
    let num_validators: usize = nodes.len() / 4;
    let total_stake: f64 = nodes.iter().map(|node| node.stake).sum();
    let mut rng = rand::thread_rng();
    let mut validator_nodes = Vec::new();

    for _ in 0..num_validators {
        let mut stake_total = 0.0;
        let target: f64 = rng.gen_range(0.0..total_stake);
        for node in nodes.iter() {
            stake_total += node.stake;
            if stake_total >= target {
                validator_nodes.push(node.clone());
                break;
            }
        }
    }
    println!("{:?}", validator_nodes);
    validator_nodes.sort_by(|a, b| a.stake.total_cmp(&b.stake));
    println!("{:?}", validator_nodes);
    let leaders: Vec<String> = validator_nodes[0..3].to_vec()
        .iter().map(|a| a.id.clone()).collect();
    let validators: Vec<String> = validator_nodes[0..num_validators].to_vec().iter()
        .map(|a| a.id.clone()).collect();

    println!("{:?}", leaders);
    (validators, leaders)
}