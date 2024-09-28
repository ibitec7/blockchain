use rand::Rng;
use crate::node::Node;

pub trait PoS {
    fn propose_stake(&mut self);

    fn select_validators(nodes: Vec<Node>, num_validators: usize) -> Vec<String>;
}

pub trait Pbft {
    fn preprepare_phase(validators: &mut Vec<Node>){}

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

    fn select_validators(nodes: Vec<Node>, num_validators: usize) -> Vec<String> {
        let total_stake: f64 = nodes.iter().map(|node| node.stake).sum();
        let mut rng = rand::thread_rng();
        let mut validators = Vec::new();

        for _ in 0..num_validators {
            let mut stake_total = 0.0;
            let target: f64 = rng.gen_range(0.0..total_stake);
            for node in nodes.iter() {
                stake_total += node.stake;
                if stake_total >= target {
                    validators.push(node.id.clone());
                    break;
                }
            }
        }
        validators
    }
}