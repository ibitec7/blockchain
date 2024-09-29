use bls_signatures::{PrivateKey, PublicKey};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::vec::Vec;
use std::sync::MutexGuard;
use std::time::{ Duration, UNIX_EPOCH };
use tokio::time;
use rand::{distributions::{Alphanumeric, Uniform}, Rng};
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::SystemTime;
use crate::transaction::{ Transaction, generate_key_pair};

#[derive(Clone)]
pub struct User {
    pub user_id: String,
    pub balance: f64,
    private_key: PrivateKey,
    pub public_key: PublicKey,
}

impl User {
    fn generate_id() -> String {
        let rng = rand::thread_rng();
        rng.sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    }

    pub fn new() -> Self{
        let id = User::generate_id();
        let (private, public) = generate_key_pair();
        User { user_id: id, balance: 1000.0, private_key: private, public_key: public }
    }

    pub fn sign_transaction(&self, tx: &mut Transaction) {
        tx.sign_transaction(self.private_key);
    }

    pub async fn simulate_transaction (&self, producer: &FutureProducer, user_base: Vec<User>, kafka_topic: String) {
        let index = user_base.iter().position(|user| { user.user_id == self.user_id }).unwrap();
        let user_dist: Uniform<usize> = Uniform::new(0, user_base.len());
        let mut rng = {
            let rng = rand::thread_rng();
            StdRng::from_rng(rng).unwrap()
        };
        let mut to_index = rng.sample(user_dist);
        while index == to_index {
            to_index = rng.sample(user_dist);
        }

        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let amount_dist: Uniform<f64> = Uniform::new(0.0, 120.0);
        let amount: f64 = rng.sample(amount_dist);
        let fees: f64 = 0.01 * amount;
        let to = user_base[to_index].user_id.clone();

        let mut transaction = Transaction::new(self.user_id.clone(), to, time, amount, fees);
        transaction.sign_transaction(self.private_key);

        let record_json = transaction.serialize();

        let record = FutureRecord::to(&kafka_topic)
            .payload(&record_json)
            .key("transaction data");

        producer.send(record, Duration::from_secs(0)).await.unwrap();

        time::sleep(Duration::from_secs(1)).await;
    }
}