use rand::{distributions::Uniform, rngs::StdRng, SeedableRng};
use rdkafka::{producer::{BaseProducer,  BaseRecord}, ClientConfig};
use simulate::User;
use rand::Rng;
use std::time::Instant;
use std::time::Duration;
use serde::Deserialize;
use serde_yaml;
use std::thread;
use tokio::fs;
use crate::transaction::Transaction;

pub mod transaction;
pub mod test_transaction;
pub mod simulate;

#[derive(Deserialize)]
pub struct ProducerConfig {
    pub server: String,
    pub autocommit: String,
    pub batchsize: String,
    pub lingerms: String,
    pub compressiontype: String,
    pub acks: String
}

#[derive(Deserialize)]
pub struct Config {
    pub user_thro: u64,
    pub user_size: usize,
    pub tx_size: u64,
    producer: ProducerConfig
}

pub async fn load_config() -> Option<Config> {
    let path = "src/config.yaml";

    let config_content = fs::read_to_string(&path).await.expect("Failed to read file");

    let config: Config =serde_yaml::from_str(&config_content).expect("Failed to parse yaml file");

    Some(config)
}

#[tokio::main]
async fn main() {
    let mut user_base = vec![];
    let mut user_base_str = vec![];
    let mut user_base_rec = vec![];
    let mut user_ids: Vec<String> = vec![];
    let topic = String::from("Users");
    let config = load_config().await.unwrap();

    for _ in 0..config.user_size{
        let user = User::new();
        let record_json = user.serialize();
        user_base_str.push(record_json);
        user_ids.push(user.user_id.clone());
        user_base.push(user);
    }

    for i in 0..config.user_size {
        let record = BaseRecord::to(&topic)
            .payload(&user_base_str[i])
            .key("User data");
        user_base_rec.push(record);
    }

    let dist = Uniform::new(0, user_base.len() - 1);
    let producer: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.producer.server)
        .set("acks", &config.producer.acks)
        .set("group.id", "sim")
        .set("linger.ms", &config.producer.lingerms)
        .set("batch.size", &config.producer.batchsize)
        .set("compression.type", &config.producer.compressiontype)
        .set("enable.auto.commit", &config.producer.autocommit)
        .create()
        .expect("Failed to create producer");

    let producer2: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.producer.server)
        .set("acks", &config.producer.acks)
        .set("group.id", "sim")
        .set("linger.ms", &config.producer.lingerms)
        .set("batch.size", &config.producer.batchsize)
        .set("compression.type", &config.producer.compressiontype)
        .set("enable.auto.commit", &config.producer.autocommit)
        .create()
        .expect("Failed to create producer");


    for user in user_base_rec {
        producer.send(user).expect("Failed to send user data");
        thread::sleep(Duration::from_millis(20))
    }

    let mut rng = {
        let rng = rand::thread_rng();
        StdRng::from_rng(rng).unwrap()
    };

    // thread::sleep(Duration::from_secs(5));

    // let mut transaction_batch: Vec<Transaction> = vec![];
    // let mut transactions: Vec<String> = vec![];
    // let mut tx_record = vec![];
    let kafka_topic = String::from("Transactions");

    thread::sleep(Duration::from_secs(5));

    let mut transaction_batch: Vec<Transaction> = vec![];
    let mut transactions: Vec<String> = vec![];    
    
    for _ in 0..config.tx_size {
        let idx = rng.sample(dist);
        transaction_batch.push(user_base[idx].simulate_transaction(user_ids.clone()));
        if transaction_batch.len() % 64 == 0 {
            let record = serde_json::to_string(&transaction_batch).expect("Failed to serialize");
            transactions.push(record);
            transaction_batch = vec![];
        }
    }

    let start = Instant::now();

    for i in 0..transactions.len(){
        let record = BaseRecord::to(&kafka_topic)
        .payload(&transactions[i])
        .key("transaction data");
        match producer2.send(record){
            Ok(_) => {},
            Err(_) => {panic!["Failed to send message"]}
        };

        thread::sleep(Duration::from_millis(10));        
    } 

    let thro = ((config.tx_size as f64) / (start.elapsed().as_millis() as f64)) * 1000.0;
    println!("Throughput: {}", thro);
}