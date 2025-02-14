// use bls_signatures::{PrivateKey, Serialize};
use rand::{distributions::Uniform, rngs::StdRng, Rng, SeedableRng};
use std::time::SystemTime;
use ring::signature::Ed25519KeyPair;
use std::time::UNIX_EPOCH;
use serde_json::{to_string, from_str};
use serde::{Serialize as SerdeSerialize,Deserialize};

use crate::transaction::{ Transaction, generate_key_pair };

pub struct User {
    pub user_id: String,
    pub balance: f64,
    private_key: Ed25519KeyPair,
}

#[derive(SerdeSerialize, Deserialize)]
pub struct UserMessage {
    pub user_id: String,
    pub balance: f64,
}

impl UserMessage {
    pub fn serialize(&self) -> String{
        let json_string = to_string(&self).unwrap();
        json_string
    }

    pub fn deserialize(json_string: &String) -> Self {
        let msg: UserMessage = from_str(json_string).unwrap();
        msg
    }
}

impl User {
    pub fn new() -> Self {
        let (private, public) = generate_key_pair();
        User { user_id: public, balance: 42000.0, private_key: private }
    }

    pub fn serialize(&self) -> String {
        let message = UserMessage { user_id: self.user_id.clone(), balance: self.balance };
        message.serialize()
    }

    pub fn sign_transaction(&self, tx: &mut Transaction){
        tx.sign_transaction(&self.private_key);
    }

    pub fn simulate_transaction(&self, user_base: Vec<String>) -> Transaction {
        let index = user_base.iter().position(|user| { *user == self.user_id }).unwrap();
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
        let to = user_base[to_index].clone();

        let mut transaction = Transaction::new(self.user_id.clone(), to, time, amount, fees);
        transaction.sign_transaction(&self.private_key);

        return transaction;
    }
}