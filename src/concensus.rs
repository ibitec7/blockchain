use rand::Rng;
use crate::{block::Block, network::Network, node::Node, transaction::Transaction};
use crate::network::{NodeMessage, MessageType};

pub trait PoS {
    fn propose_stake(&mut self);

    fn select_validators(&mut self, nodes: Vec<Node>);
}

pub trait Pbft {
    fn preprepare_phase<T: Pbft + Network> (&mut self, pool: Vec<Transaction>){}

    fn prepare_phase(){}

    fn commit_phase(){}

    fn reply_phase(){}
}

impl PoS for Node {
    fn propose_stake(&mut self){
        let dist = rand::distributions::Uniform::new(10.0, 500.0);
        let mut rng = rand::thread_rng();
        self.stake = rng.sample(dist);
    }

    fn select_validators(&mut self ,nodes: Vec<Node>) {
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
        validator_nodes.sort_by(|a, b| a.stake.total_cmp(&b.stake).reverse());
        println!("{:?}", validator_nodes);
        let leaders: Vec<String> = validator_nodes[0..2].to_vec()
            .iter().map(|a| a.id.clone()).collect();
        let validators: Vec<String> = validator_nodes[0..num_validators].to_vec().iter()
            .map(|a| a.id.clone()).collect();

        println!("{:?}", leaders);
        self.validators = validators;
        self.primary = leaders;
    }
}

impl Pbft for Node {
    fn preprepare_phase<T: Pbft + Network> (&mut self, pool: Vec<Transaction>) {
        self.staging = pool.clone();
        println!("prev_hash: {:?}", hex::encode(
            self.block_chain.chain[self.block_chain.chain.len() - 1].hash()));
        println!("index: {:?}", self.block_chain.chain[self.block_chain.chain.len() - 1].index + 1);
        let block = Block::new(pool, 
            hex::encode(
            self.block_chain.chain[self.block_chain.chain.len() - 1].hash()), 
            self.block_chain.chain[self.block_chain.chain.len() - 1].index + 1);

        let message = NodeMessage { msg_type: MessageType::Request(block.serialize_block()), sender_id: self.id.clone(), seq_num: 1 };

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
}