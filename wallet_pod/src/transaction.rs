use serde::Serialize;
use serde_json;
use std::collections::HashMap;

#[derive(Serialize, Debug,Clone)]
pub struct Output {
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
pub struct UTXO {
    pub id: String,             // The unique id of the UTXO
    pub input: Vec<String>,     // The input UTXOs for this transactions
    pub output: Vec<Output>,    // The output UTXOs coming from this transactions
    pub amount: u64,            // The amount??
    pub owner: Script           // The ownership script that will allow the UTXO to be redeemed
}

impl UTXO {
    fn new(id: String, amount: u64, input: Vec<String>,
        output: Vec<Output>, owner: Script) -> Self {

        UTXO { id, input, output, amount, owner }
    }

    fn serialize(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize")
    }
}