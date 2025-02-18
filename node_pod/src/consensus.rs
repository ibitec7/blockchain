use bls_signatures::{PublicKey, Serialize};
use rdkafka::producer::{BaseProducer, BaseRecord};
use std::collections::HashMap;
use rand::Rng;
use serde_json::{from_str, to_string};
use crate::definitions::{consensus_header::StakeMethods, network_header::{MessageType, MessageTypeMethods, Network, NodeMessage, NodeMessageMethods}, node_header::Node, transaction_header::Transaction};
use crate::definitions::block_header::{Block, BlockMethods, BlockChainMethods};
use crate::definitions::consensus_header::{PoS, Pbft, Stake, Validator, ValidatorMethods};

impl StakeMethods for Stake {
    fn new(node: &Node, stake: f64) -> Self {
        Stake { node_id: node.id.clone() , stake }
    }

    fn serialize(&self) -> String {
        let json_string = to_string(&self).expect("Failed to serialize");
        json_string
    }

    fn deserialize(json_str: String) -> Self {
        let stake: Stake = from_str(&json_str).expect("Failed to deserialize");
        stake
    }
}

impl ValidatorMethods for Validator {
    fn serialize(&self) -> String {
        let json_string = to_string(&self).expect("Failed to serialize");
        json_string
    }

    fn deserialize(json_str: String) -> Self {
        let validator: Validator = from_str(&json_str).expect("Failed to deserialize");
        validator
    }
}

impl PoS for Node {
    async fn propose_stake(&mut self, producer: &BaseProducer){
        let dist = rand::distributions::Uniform::new(10.0, 500.0);
        let mut rng = rand::thread_rng();
        self.stake = rng.sample(dist);

        let stake = Stake::new(self, self.stake);
        let record_json = StakeMethods::serialize(&stake);
        let topic = String::from("Stakes");
        let record = BaseRecord::to(&topic)
            .payload(&record_json)
            .key("Node Stake");

        producer.send(record).expect("Failed to send the stake");
    }
}

impl Pbft for Node {
    async fn preprepare_phase (&mut self, pool: Vec<Transaction>, producer: &BaseProducer) {

        self.staging = pool.clone();

        let block = Block::new(pool,
            self.block_chain.chain[self.block_chain.chain.len() - 1].hash.clone(), 
            self.block_chain.chain[self.block_chain.chain.len() - 1].index + 1);

        let primary = self.primary.clone();
        let id = self.id.clone();
        let mut is_primary = false;
        for leader in primary {
            if leader.node_id == id {
                is_primary = true
            } else { continue; }
        }
        if is_primary {
            let message = NodeMessage::new(self, &block, String::from("Preprepare"), self.msg_idx[0].clone());

            self.block_staging.push(block);

            self.broadcast_kafka("Preprepare", message, producer).await;
        }
        else {
            self.block_staging.push(block);
        }

        self.msg_idx[0] += 1;
    }

    async fn prepare_phase(&mut self, pkey_store: &HashMap<String, PublicKey>, primary_msg: Vec<String>, producer: &BaseProducer) {
        
        let mut messages: Vec<NodeMessage> = vec![];
        
        for msg in primary_msg {
            messages.push(NodeMessage::deserialize_message(msg));
        }

        let val = self.validators.clone();
        let id = self.id.clone();
        let mut is_val = false;
        for validator in val {
            if validator.node_id == id {
                is_val = true
            } else { continue; }
        }

        if !is_val {
            return;
        }

        for msg in messages {
            let is_leader: bool = self.primary.iter().any(|validator| validator.node_id == msg.sender_id);
            let is_preprepare = match msg.msg_type {
                MessageType::PrePrepare(_) => true,
                _ => false
            };

            let pkey = pkey_store.get(&msg.sender_id).expect("Sender is not in Key Store");
            let verify_leader = pkey.verify(bls_signatures::Signature::from_bytes(
                &hex::decode(msg.signature).unwrap()).unwrap()
                , msg.msg_type.unwrap());

            if is_leader && is_preprepare && verify_leader {
                let new_block: Block = Block::deserialize_block(&msg.msg_type.unwrap());
                let verify_block: bool = self.block_staging.last().unwrap().is_equal(new_block);

                if verify_block {
                    let kafka_message: NodeMessage = NodeMessage::new(self,
                        &self.block_staging[msg.seq_num.clone()].clone(), String::from("Prepare"), self.msg_idx[1]);

                    self.broadcast_kafka("Prepare", kafka_message, producer).await;

                    self.msg_idx[1] += 1;
                }
                else {
                    panic!["Cant verify block"]
                }
            }
            else if !is_leader {
                panic!["cant make sure is leader"]
            }
            else if !is_preprepare {
                panic!["cant is preprepare"]
            }
            else if !verify_leader {
                panic!["cant verify leader"]
            }
        }
    }

    async fn commit_phase(&mut self, pkey_store: &HashMap<String, PublicKey>, prepare_msg: Vec<String>, producer: &BaseProducer) -> bool {

        let mut messages: Vec<NodeMessage> = vec![];

        // Count the number of blocks that are same
        // if a new block is seen then push a new count 1
        // ideally its dimension should be 1
        let mut blocks:Vec<Block> = vec![];
        let mut counts: Vec<i32> = vec![];

        let mut a: f64 = 0.0;
        let mut b: f64 = 0.0;

        for msg in prepare_msg {
            messages.push(NodeMessage::deserialize_message(msg));
            a += 1.0;
        }

        for _ in 0..self.validators.len() {
            b += 1.0;
        }

        let faulty_nodes: f64 = b - a;
        let threshold: f64 = (b - 1.0) / 3.0;

        let exceed_thresh: bool = faulty_nodes > threshold;

        if exceed_thresh {
            panic!["Concensus failed! threshold exceeded"]
        }

        for msg in messages {
            let is_validator = self.validators.iter().any(|validator| validator.node_id == msg.sender_id);
            let is_prepare = match msg.msg_type {
                MessageType::Prepare(_) => true,
                _ => false
            };
            let pkey = pkey_store.get(&msg.sender_id).expect("Sender is not in the key store");
            let verify_sender = pkey.verify(bls_signatures::Signature::from_bytes(
                &hex::decode(msg.signature).unwrap()).unwrap()
                , msg.msg_type.unwrap());

            let new_block: Block = Block::deserialize_block(&msg.msg_type.unwrap());
            
            if is_validator && is_prepare && verify_sender {
                if blocks.len() == 0 && new_block.validate(new_block.transactions.clone()) {
                    blocks.push(new_block);
                    counts.push(1);
                }
                else if blocks.len() != 0 {
                    let mut found = false;
                    for i in 0..blocks.len() {
                        if blocks[i].is_equal(new_block.clone()){
                            counts[i] += 1;
                            found = true;
                        }
                    }
                    if !found && new_block.validate(new_block.transactions.clone()) {
                        blocks.push(new_block.clone());
                        counts.push(1);
                    }
                }
            }
            else if !is_validator {
                panic!["sender is not the validator"];
            }
            else if !is_prepare {
                panic!["Message is not prepare"];
            }
            else if !verify_sender {
                panic!["Sender can not be verified"];
            }
        }

        let max_count = match counts.iter().max() {
            Some(max) => {max},
            Option::None => panic!["Can not find max in {:?}", counts]
        };

        let max_idx = counts.iter().position(|x| x == max_count).unwrap();

        let mut a: i32 = 0;
        for i in 0..counts.len() {
            if counts[i] == *max_count {
                a += 1;
            }
        }

        if a != 1 {
            panic!["Concensus Not reached more than one majority on different blocks"];
        }
        else {

        }

        let new_block: Block = blocks[max_idx].to_owned();


        let kafka_message: NodeMessage = NodeMessage::new(self,
            &new_block.clone(), String::from("Commit"), self.msg_idx[2]);

        self.broadcast_kafka("Commit", kafka_message, producer).await;

        self.msg_idx[2] += 1;

        self.block_chain.add_block(new_block);
        
        self.block_chain.verify_chain()

    }
}