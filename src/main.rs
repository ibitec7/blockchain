use std::ops::Deref;
use std::sync::Arc;
use futures_util::lock::Mutex;
use node::Miner;
use tokio::sync::Barrier;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::{distributions::Uniform, Rng};
use rdkafka::{producer::FutureProducer, ClientConfig};
use simulate::User;
use lazy_static::lazy_static;
use crate::block::BlockChain;
use crate::node::Node;
use crate::concensus::PoS;

pub mod concensus;
pub mod network;
pub mod transaction;
pub mod node;
pub mod merkle_tree;
pub mod block;
pub mod simulate;

pub async fn simulate_tx(user_base: Arc<Vec<User>>) {
    let dist = Uniform::new(0, user_base.len() - 1);
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Failed to create producer");
    let mut rng = {
        let rng = rand::thread_rng();
        StdRng::from_rng(rng).unwrap()
    };

    for _ in 0..1000 {
        let index = rng.sample(dist);
        user_base[index].simulate_transaction(&producer, user_base.deref().clone(), String::from("Transactions")).await;
    }
}

pub async fn listen(node: &mut Node, topics: &[&str], users: Arc<Vec<User>>, miner: &Miner) {
    for _ in 0..4 {
        node.pool_transactions(topics, users.deref().clone(), miner).await;
    }
}

#[tokio::main]
async fn main() {
    let chain = BlockChain::new();
    let topics: Vec<&str> = vec!["Transactions"];
    let mut node_network: Vec<Node> = vec![];

    // Create the nodes in the network
    for _ in 0..20 {
        let mut node = Node::new(chain.clone());
        node.propose_stake();
        node_network.push(node.clone());
    }

    let network_clone = node_network.clone();

    let mut miners_init = vec![];
    for node in node_network {
        miners_init.push(Miner::new(&node))
    }

    lazy_static!(
        static ref miners = miners_init.clone();
    )

    let (validator_nodes, primary_nodes) = concensus::select_validators(network_clone);

    let node_network: Vec<Node> = node_network.iter_mut().map(|node| 
        {
            node.set_validators(validator_nodes.clone(), primary_nodes.clone());
            node.to_owned()
        }).collect();

    let mut user_base = vec![];
    for _ in 0..100 {
        user_base.push(User::new());
    }

    let user_base = Arc::new(user_base);
    let barrier = Arc::new(Barrier::new(node_network.len() + 1));

    let user_base_clone = Arc::clone(&user_base);

    println!("Making simulate handle");
    
    let simulate_handle = tokio::spawn(async move {
        simulate_tx(user_base_clone).await;
    });

    let mut handles = vec![];

    println!("Making node network handles");
    for mut node in node_network {
        let barrier_clone = Arc::clone(&barrier);
        let user_base_clone = Arc::clone(&user_base);
        let topics_clone = topics.clone();
        let miners_ref = miners.as_ref()

        
        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            listen(&mut node, &topics_clone, user_base_clone, miner).await;
        });

        handles.push(handle);
    }

    println!("Waiting for barrier to drop");
    barrier.wait().await;
    println!("Barrier down now executing...");

    for handle in handles {
        handle.await.unwrap();
    }

    println!("Executing simulation");

   
    simulate_handle.await.unwrap();
}
