use ring::signature;
use serde::Serialize;
use serde_json;
use std::time::{SystemTime, Duration};
use std::collections::HashMap;

#[derive(Serialize, Debug,Clone)]
pub struct UTXO {
    pub id: String,
    pub amount: u64,
    pub to: String
}

#[derive(Serialize, Debug, Clone)]
pub struct Script {
    pub n_keys: u32,
    pub min_keys: u32,
    pub pub_keys: Vec<String>,
    pub signatures: HashMap<String, String>
}

#[derive(Serialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,             // The unique id of the Transaction
    pub timestamp: u64,         // The timestamp of the transaction
    pub input: Vec<String>,     // The input Transactions for this transactions
    pub utxo: Vec<UTXO>,        // The UTXO Transactions coming from this transactions
    pub amount: u64,            // The amount??
    pub owner: Script           // The ownership script that will allow the Transaction to be redeemed
}

impl Script {
    pub fn new(n_keys: u32, min_keys: u32, pub_keys: Vec<String>) -> Script{

        Script {
            n_keys,
            min_keys,
            pub_keys,
            signatures: HashMap::new()
        }

    }
}

impl Transaction {
    pub fn new(id: String, timestamp: u64 , amount: u64, input: Vec<String>,
        utxo: Vec<UTXO>, owner: Script) -> Self {

        Transaction { id, timestamp, input, utxo, amount, owner }

    }

    fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize")
    }
}