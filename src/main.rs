use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::Barrier;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::{distributions::Uniform, Rng};
use rdkafka::{producer::FutureProducer, ClientConfig};
use simulate::User;
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

pub async fn listen(node: &mut Node, topics: &[&str], users: Arc<Vec<User>>) {
    for _ in 0..4 {
        node.pool_transactions(topics, users.deref().clone()).await.unwrap();
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

    // Select validators for each node
    for mut node in node_network.clone() {
        node.select_validators(node_network.clone());
    }

    let mut user_base = vec![];
    for _ in 0..100 {
        user_base.push(User::new());
    }

    let user_base = Arc::new(user_base);
    let barrier = Arc::new(Barrier::new(node_network.len() + 1));

    let barrier_clone = Arc::clone(&barrier);
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
        
        let handle = tokio::spawn(async move {
            barrier_clone.wait().await;
            listen(&mut node, &topics_clone, user_base_clone).await;
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
