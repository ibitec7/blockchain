use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{BaseProducer, BaseRecord};
use stake::Validator;
use rdkafka::ClientConfig;
use std::time::Duration;
use serde::Deserialize;
use rdkafka::Message;
use rand::Rng;
use log::info;
use rand_distr::{Distribution, WeightedIndex};
use futures_util::stream::StreamExt;
use tokio::time::timeout;
use std::collections::HashSet;
use tokio::fs;
use std::thread;

pub mod stake;
use crate::stake::Stake;

#[derive(Deserialize)]
pub struct ConsumerConfig {
    pub server: String,
    pub autocommit: String,
    pub autooffset: String,
    pub acks: String
}

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
pub struct PerformanceConfig {
    pub timeout: u64,
}

#[derive(Deserialize)]
pub struct StakingConfig {
    pub validators: usize
}

#[derive(Deserialize)]
pub struct Config {
    consumer: ConsumerConfig,
    producer: ProducerConfig,
    performance: PerformanceConfig,
    staking: StakingConfig
}

pub async fn load_config() -> Option<Config> {
    info!("Loading config");

    let path = "src/config.yaml";

    let config_content = fs::read_to_string(&path).await.expect("Failed to read file");

    let config =serde_yaml::from_str(&config_content).expect("Failed to parse yaml file");

    config
}


pub async fn listen_stake(consumer: &StreamConsumer, time_out: u64,
vals: &usize) -> Option<Vec<Stake>> {

    info!("Listening for the stakes");
    let mut stakes: Vec<Stake> = vec![];

    let mut msg_stream = consumer.stream();

    loop {
        match timeout(Duration::from_millis(time_out), msg_stream.next()).await {
        Ok(Some(message_result)) => {
            match message_result {
                Err(e) => eprintln!("Error while receiving message: {}", e),
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        match serde_json::from_slice::<Stake>(payload){
                            Ok(user) => {
                                stakes.push(user);
                            }
                            Err(e) => {
                                eprintln!("Failed to deserialize message: {}", e);
                            }
                        }
                    }
                }
            }
        }
        Ok(_) => {
            panic!["stream ended"];
        }
        Err(_) => {
            if stakes.len() == *vals {
                break;
            } else if stakes.len() < *vals && !stakes.is_empty() {
                msg_stream = consumer.stream();
                continue;
            } else {
                continue;
            }
        }
        }
    }
    if stakes.is_empty() {
        return None;
    }
    else {
        Some(stakes)
    }
}

fn validator_selection(
    stakes: &[Stake],
    vals: usize,
    producer: &BaseProducer,
    producer2: &BaseProducer,
    time_out: u64,
) {

    info!("Selecting validators");

    let weights: Vec<f64> = stakes.iter().map(|s| s.stake).collect();
    let mut rng = rand::thread_rng();
    let dist = WeightedIndex::new(&weights).expect("Invalid weights");

    let mut selected_indices = HashSet::new();
    let mut validators: Vec<Validator> = Vec::new();

    while validators.len() < vals {
        let index = dist.sample(&mut rng);

        if selected_indices.contains(&index) {
            continue; 
        }

        selected_indices.insert(index);
        validators.push(Validator::from_stake(&stakes[index]));
    }

    let primary_index = rng.gen_range(0..validators.len());
    let primary = validators[primary_index].clone();

    for val in &validators {
        let record_json = val.serialize();
        info!("Validator: {}", record_json);
        producer
            .send(
                BaseRecord::to("Validators")
                    .payload(&record_json)
                    .key(&val.node_id),
            )
            .expect("Failed to send validator message");
    }

    thread::sleep(Duration::from_millis(time_out));

    let p_record_json = primary.serialize();
    producer2
        .send(
            BaseRecord::to("Primary")
                .payload(&p_record_json)
                .key(&primary.node_id),
        )
        .expect("Failed to send primary message");
}

#[tokio::main]
async fn main() {
    let config = load_config().await.unwrap();

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", config.consumer.server)
        .set("group.id", "master")
        .set("enable.auto.commit",&config.consumer.autocommit)
        .set("auto.offset.reset", &config.consumer.autooffset)
        .set("acks", &config.consumer.acks)
        .create()
        .expect("Failed to create stream consumer");

    let producer: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.producer.server)
        .set("group.id", "master_id")
        .set("acks", &config.producer.acks)
        .set("linger.ms", &config.producer.lingerms)
        .set("batch.size", &config.producer.batchsize)
        .set("compression.type", &config.producer.compressiontype)
        .set("enable.auto.commit",&config.producer.autocommit)
        .create()
        .expect("Failed to create producer");

    let producer2: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.producer.server)
        .set("group.id", "master_id")
        .set("acks", &config.producer.acks)
        .set("linger.ms", &config.producer.lingerms)
        .set("batch.size", &config.producer.batchsize)
        .set("compression.type", &config.producer.compressiontype)
        .set("enable.auto.commit",&config.producer.autocommit)
        .create()
        .expect("Failed to create producer");

    consumer.subscribe(&["Stakes"]).expect("Failed to subscribe to topic");
    
    loop {
        let stakes: Option<Vec<Stake>> = listen_stake(&consumer, config.performance.timeout, &config.staking.validators).await;

        match stakes {
            Option::None => continue,
            Some(stakes_vec) => {
                info!("Stakes received");
                validator_selection(&stakes_vec.as_slice(), config.staking.validators, &producer,
            &producer2,config.performance.timeout);
                info!("Validators selected");
        },
        }
    }
}