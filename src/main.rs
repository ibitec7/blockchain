use block::BlockChain;
use node::Node;
use rand::{distributions::Uniform, Rng};
use rdkafka::{producer::FutureProducer, ClientConfig};
use simulate::User;

pub mod concensus;
pub mod network;
pub mod transaction;
pub mod node;
pub mod merkle_tree;
pub mod block;
pub mod simulate;

pub async fn simulate_tx(user_base: Vec<User>) {
    let dist = Uniform::new(0, user_base.len() - 1);
    let mut rng = rand::thread_rng();
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("Failed to make producer");
    for _ in 0..1000 {
        let index = rng.sample(dist);
        user_base[index].simulate_transaction(&producer, user_base.clone(), String::from("Transactions")).await;
    }
}

pub async fn listen(node1: &mut Node,topics: &[&str], users: Vec<User>) {
    for _ in 0..4 {
        node1.pool_transactions(topics, users.clone()).await.unwrap();
    }
}


#[tokio::main]
async fn main(){
    let chain = BlockChain::new();
    let topics: Vec<&str> = vec!["Transactions"];
    let mut node1 = Node::new(chain.clone());
    let mut node2 = Node::new(chain.clone());
    let mut user_base: Vec<User> = Vec::new();
    for _ in 0..100 {
        user_base.push(User::new());
    }
    
    let (_, _, _) = tokio::join!(simulate_tx(user_base.clone()), 
        listen(&mut node1, topics.as_slice(), user_base.clone()),
        listen(&mut node2, topics.as_slice(), user_base.clone()));

    
}
