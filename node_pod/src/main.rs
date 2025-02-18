use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::BaseProducer;
use rdkafka::ClientConfig;
use simulate::User;
use std::time::Duration;
use rdkafka::Message;
use futures_util::stream::StreamExt;
use tokio::time::{timeout, Instant};
use crate::node_header::{ConcensusMetrics, PoolingMetrics, Node, NodeMethods};
use std::sync::Arc;
use tokio::fs;
use serde_yaml;
use serde::{Serialize as SerdeSerialize, Deserialize};
use crate::transaction_header::Transaction;
use bls_signatures::{PublicKey, Serialize};
use std::collections::HashMap;
use crate::consensus_header::{PoS, Validator};
use csv::Writer;

///     WORK ON THE STEPS THAT WILL BE DONE BY THE NODES
///     IN PARTICULAR HOW THEY WILL LISTEN FOR VALIDATORS TO UPDATE THE VALIDATORS LIST AND PRIMARIES
///     ALSO HOW THEY WILL LISTEN FOR USERS TO GET USER PUB KEYS

pub mod consensus;
pub mod network;
pub mod node_header;
pub mod transaction;
pub mod consensus_header;
pub mod simulate;
pub mod test_merkle;
pub mod transaction_header;
pub mod merkle_header;
pub mod merkle_tree;
pub mod network_header;
pub mod block_header;
pub mod node;
pub mod block;

#[derive(SerdeSerialize, Deserialize, Clone)]

pub struct ConsumerConfig {
    pub server: String,
    pub autocommit: String,
    pub autooffset: String,
    acks: String
}

#[derive(SerdeSerialize, Deserialize, Clone)]
pub struct ProducerConfig {
    pub server: String,
    pub autocommit: String,
    pub batchsize: String,
    pub lingerms: String,
    pub compressiontype: String,
    pub acks: String
}

#[derive(SerdeSerialize, Deserialize, Clone)]
pub struct PerformanceConfig {
    pub tx_time: u64,
    pub timeout: u64,
    pub block_size: usize
}

#[derive(SerdeSerialize, Deserialize, Clone)]
pub struct Config{
    pub consumer: ConsumerConfig,
    pub producer: ProducerConfig,
    pub performance: PerformanceConfig
}

pub async fn listen_user(consumer: &StreamConsumer, time_out: &u64) -> Vec<User> {
    let mut users: Vec<User> = vec![];

    let mut msg_stream = consumer.stream();

    loop {
        match timeout(Duration::from_millis(*time_out), msg_stream.next()).await {
        Ok(Some(message_result)) => {
            match message_result {
                Err(e) => eprintln!("Error while receiving message: {}", e),
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        match serde_json::from_slice::<User>(payload){
                            Ok(user) => {
                                users.push(user);
                            }
                            Err(e) => {
                                eprintln!("Failed to deserialize message: {}", e);
                            }
                        }
                        if let Err(e) = consumer.commit_message(&message, rdkafka::consumer::CommitMode::Sync) {
                            eprintln!("Failed to commit message: {}", e);
                        }
                    }
                }
            }
        }
        Ok(_) => {
            break;
        }
        Err(_) => {
            if !users.is_empty(){
                break;
            } else {
                continue;
            }
        }
        }
    }
    users
}

pub async fn listen_validators(consumer: &StreamConsumer, time_out: &u64) -> Vec<Validator> {
    let mut validators: Vec<Validator> = vec![];

    let mut msg_stream = consumer.stream();

    loop {
        match timeout(Duration::from_millis(*time_out), msg_stream.next()).await {
        Ok(Some(message_result)) => {
            match message_result {
                Err(e) => eprintln!("Error while receiving message: {}", e),
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        match serde_json::from_slice::<Validator>(payload){
                            Ok(validator) => {
                                validators.push(validator);
                            }
                            Err(e) => {
                                eprintln!("Failed to deserialize message: {}", e);
                            }
                        }
                        if let Err(e) = consumer.commit_message(&message, rdkafka::consumer::CommitMode::Sync) {
                            eprintln!("Failed to commit message: {}", e);
                        }
                    }
                }
            }
        }
        Ok(_) => {
            break;
        }
        Err(_) => {
            if !validators.is_empty() {
                break;
            } else {
                continue;
            }
        }
        }
    }
    validators
}

pub async fn load_config() -> Option<Config> {
    let path = "src/config.yaml";

    let config_content = fs::read_to_string(&path).await.expect("Failed to read file");

    let config =serde_yaml::from_str(&config_content).expect("Failed to parse yaml file");

    config
}

#[derive(SerdeSerialize)]
pub struct Record {
    pub pool_tps: f64,
    pub pool_process_time: f64,
    pub failed_transactions: f64,
    pub ttf: f64,
    pub staking_time: f64,
    pub preprepare_time: f64,
    pub preprepare_wait: f64,
    pub prepare_time: f64,
    pub prepare_wait: f64,
    pub commit_time: f64,
    pub commit_wait: f64,
    pub block_tps: f64,
    pub concensus_time: f64,
    pub total_time: f64,
}

#[tokio::main]
async fn main() {
    let mut node = Node::new();

    let config = load_config().await.unwrap();

    let mut wtr: Writer<_> = Writer::from_path("data.csv").expect("can not find file path");
    // wtr.write_record(&["pool_tps", "pool_process_time", "failed_transactions", "ttf", "preprepare_time",
    //                 "prepare_time", "commit_time", "block_tps", "concensus_time", "total_time"]).expect("failed to write to CSV");

    let stake_producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.producer.server)
            .set("group.id", &node.id)
            .set("enable.auto.commit",&config.producer.autocommit)
            .set("linger.ms", &config.producer.lingerms)
            .set("batch.size", &config.producer.batchsize)
            .set("compression.type", &config.producer.compressiontype)
            .set("acks", &config.producer.acks)
            .create()
            .expect("Failed to create producer");

    let user_consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &config.consumer.server)
        .set("group.id", &(node.id.clone()+"user"))
        .set("enable.auto.commit",&config.consumer.autocommit)
        .set("auto.offset.reset", &config.consumer.autooffset)
        .set("acks", &config.consumer.acks)
        .create()
        .expect("Failed to create stream consumer");

    user_consumer.subscribe(&[String::from("Users").as_str()]).expect("Failed to subscribe to topic");

    // Getting the users data here

    let users: Vec<User> = listen_user(&user_consumer, &config.performance.timeout).await;

    let mut user_base: HashMap<String, f64> = HashMap::new();

    for val in users.clone() {
        user_base.insert(val.user_id, val.balance);
    }

    if users.len() < 100 {
        panic!["Not all users fetched"];
    }

    let val_consumer: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", &config.consumer.server)
    .set("group.id", &(node.id.clone()+"validator"))
    .set("enable.auto.commit",&config.consumer.autocommit)
    .set("auto.offset.reset", &config.consumer.autooffset)
    .set("acks", &config.consumer.acks)
    .create()
    .expect("Failed to create stream consumer");

    val_consumer.subscribe(&["Validators"]).expect("Failed to subscribe to topic");

    // PRIMARY LOGIC GOES HERE

    let primary_consumer: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", &config.consumer.server)
    .set("group.id", &(node.id.clone()+"primary"))
    .set("enable.auto.commit",&config.consumer.autocommit)
    .set("auto.offset.reset", &config.consumer.autooffset)
    .set("acks", &config.consumer.acks)
    .create()
    .expect("Failed to create stream consumer");

    primary_consumer.subscribe(&["Primary"]).expect("Failed to subscribe to topic");

    let prepre_consumer: Arc<StreamConsumer> = Arc::new(ClientConfig::new()
    .set("bootstrap.servers", &config.consumer.server)
    .set("group.id", &(node.id.clone()+"Preprepare"))
    .set("enable.auto.commit",&config.consumer.autocommit)
    .set("auto.offset.reset", &config.consumer.autooffset)
    .set("acks", &config.consumer.acks)
    .create()
    .expect("Failed to create stream consumer"));

    prepre_consumer.subscribe(&["Preprepare"]).expect("Subscription Error");

    let prepre_ready: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", &config.consumer.server)
    .set("group.id", &(node.id.clone()+"prepre_ready"))
    .set("enable.auto.commit",&config.consumer.autocommit)
    .set("auto.offset.reset", &config.consumer.autooffset)
    .set("acks", &config.consumer.acks)
    .create()
    .expect("Failed to create stream consumer");

    prepre_ready.subscribe(&["Status"]).expect("Subscription Error");

    let pre_consumer: Arc<StreamConsumer> = Arc::new(ClientConfig::new()
    .set("bootstrap.servers", &config.consumer.server)
    .set("group.id", &(node.id.clone()+"Prepare"))
    .set("enable.auto.commit",&config.consumer.autocommit)
    .set("auto.offset.reset", &config.consumer.autooffset)
    .set("acks", &config.consumer.acks)
    .create()
    .expect("Failed to create stream consumer"));

    pre_consumer.subscribe(&["Prepare"]).expect("Subscription Error");

    let pre_ready: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", &config.consumer.server)
    .set("group.id", &(node.id.clone()+"pre_ready"))
    .set("enable.auto.commit",&config.consumer.autocommit)
    .set("auto.offset.reset", &config.consumer.autooffset)
    .set("acks", &config.consumer.acks)
    .create()
    .expect("Failed to create stream consumer");

    pre_ready.subscribe(&["Status"]).expect("Subscription Error");

    let comm_ready: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", &config.consumer.server)
    .set("group.id", &(node.id.clone()+"comm_ready"))
    .set("enable.auto.commit",&config.consumer.autocommit)
    .set("auto.offset.reset", &config.consumer.autooffset)
    .set("acks", &config.consumer.acks)
    .create()
    .expect("Failed to create stream consumer");

    comm_ready.subscribe(&["Status"]).expect("Subscription Error");

    let prepre_producer: Arc<BaseProducer> = Arc::new(ClientConfig::new()
            .set("bootstrap.servers", &config.producer.server)
            .set("group.id", "Preprepare_prod")
            .set("group.id", &node.id)
            .set("enable.auto.commit",&config.producer.autocommit)
            .set("linger.ms", &config.producer.lingerms)
            .set("batch.size", &config.producer.batchsize)
            .set("compression.type", &config.producer.compressiontype)
            .set("acks", &config.producer.acks)
            .create()
            .expect("Failed to create producer"));

    let pre_producer: Arc<BaseProducer> = Arc::new(ClientConfig::new()
            .set("bootstrap.servers", &config.producer.server)
            .set("group.id", "Pre_prod")
            .set("group.id", &node.id)
            .set("enable.auto.commit",&config.producer.autocommit)
            .set("linger.ms", &config.producer.lingerms)
            .set("batch.size", &config.producer.batchsize)
            .set("compression.type", &config.producer.compressiontype)
            .set("acks", &config.producer.acks)
            .create()
            .expect("Failed to create producer"));

    let comm_producer: Arc<BaseProducer> = Arc::new(ClientConfig::new()
            .set("bootstrap.servers", &config.producer.server)
            .set("group.id", "Comm_prod")
            .set("group.id", &node.id)
            .set("enable.auto.commit",&config.producer.autocommit)
            .set("linger.ms", &config.producer.lingerms)
            .set("batch.size", &config.producer.batchsize)
            .set("compression.type", &config.producer.compressiontype)
            .set("acks", &config.producer.acks)
            .create()
            .expect("Failed to create producer"));

    tokio::time::sleep(Duration::from_secs(1)).await;

    // POOL LOGIC GOES HERE

    let tx_consumer: StreamConsumer = ClientConfig::new()
    .set("bootstrap.servers", &config.consumer.server)
    .set("group.id", &(node.id.clone()+"tx"))
    .set("enable.auto.commit",&config.consumer.autocommit)
    .set("auto.offset.reset", &config.consumer.autooffset)
    .set("acks", &config.consumer.acks)
    .set("fetch.min.bytes", "1024")
    .create()
    .expect("Failed to create stream consumer");

    tx_consumer.subscribe(&[String::from("Transactions").as_str()]).expect("Failed to subscribe to topic");

    // CONCENSUS LOGIC GOES HERE

    let mut resid: Vec<Transaction> = vec![];
    let mut pool: Option<Vec<Transaction>>; 
    let mut pool_metrics: Option<PoolingMetrics>;
    let mut concensus_metrics: Option<ConcensusMetrics>;
    loop {
        let start1 = Instant::now();

        let (_, validators, primary) = tokio::join!(
            node.propose_stake(&stake_producer),
            listen_validators(&val_consumer, &config.performance.timeout),
            listen_validators(&primary_consumer, &config.performance.timeout)
        );

        node.validators = validators;
        node.primary = primary;

        let mut pkey_store: HashMap<String, PublicKey> = HashMap::new();

        for val in node.validators.clone() {
            pkey_store.insert(val.node_id, PublicKey::from_bytes(hex::decode(val.public_key).unwrap().as_slice()).unwrap());
        }

        let end1 = start1.elapsed().as_millis() as f64;

        (pool, resid, pool_metrics) = node.pool_transactions(&tx_consumer, &mut user_base, 
        &mut resid,config.performance.timeout, config.performance.tx_time,
        &config.performance.block_size).await;

        let pool_perf = pool_metrics.unwrap();

        let prepre_con_clone= Arc::clone(&prepre_consumer);
        let pre_con_clone= Arc::clone(&pre_consumer);
        let prepre_prod_clone= Arc::clone(&prepre_producer);
        let pre_prod_clone= Arc::clone(&pre_producer);
        let comm_prod_clone= Arc::clone(&comm_producer);
        
        let start = Instant::now();

        match pool {
            Option::None => {panic!["No pool continuing"]},
            Some(tx_pool) =>  {concensus_metrics = node.concensus(tx_pool, pkey_store,
            prepre_con_clone, pre_con_clone, &prepre_ready, &pre_ready, &comm_ready,
        prepre_prod_clone, pre_prod_clone, comm_prod_clone,config.performance.timeout).await}
        };

        let concensus_perf = concensus_metrics.unwrap();
        let end = start.elapsed().as_millis() as f64;
        // println!("Block {} is out!", node.msg_idx[0]);
        let end_total = start1.elapsed().as_millis() as f64;
        let concensus_total = end1 + end;
        let ttf = end + pool_perf.ttf;

        let block_tps = 1000.0 * ((config.performance.block_size as f64) / (end + end1));

        let record = Record {
            pool_tps: pool_perf.tps,
            pool_process_time: pool_perf.processtime,
            failed_transactions: pool_perf.bad_tx,
            ttf,
            staking_time: end1,
            preprepare_time: concensus_perf.prepre_time,
            preprepare_wait: concensus_perf.prepre_wait,
            prepare_time: concensus_perf.pre_time,
            prepare_wait: concensus_perf.pre_wait,
            commit_time: concensus_perf.commit_time,
            commit_wait: concensus_perf.commit_wait,
            block_tps,
            concensus_time: concensus_total,
            total_time: end_total
        };

        wtr.serialize(record).expect("failed to serialize record");

        wtr.flush().expect("Failed to flush wtr");
    }
}