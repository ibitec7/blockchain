use hex::ToHex;
use serde::{ Serialize, Deserialize };
use ring::{pkcs8, rand::SystemRandom,signature::{KeyPair, Ed25519KeyPair}};

#[derive(Serialize, Deserialize, Clone, std::fmt::Debug)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub timestamp: u64,
    pub amount: f64,
    pub fee: f64,
    pub signature: String,
}

pub trait TransactionMethods: Clone + serde::Serialize + for <'de> serde::Deserialize<'de> {

    fn new(from_: String, to_: String, time: u64, amt: f64, fees:f64) -> Self;

    fn serialize(&self) -> String;

    fn deserialize(json_string: &str) -> Self;

    fn sign_transaction(&mut self, private_key: &Ed25519KeyPair);

    fn generate_transaction_id(&mut self) -> String;

    fn verify_transaction(&self, public_key_str: String) -> bool;

}

pub fn generate_key_pair() -> (Ed25519KeyPair, String) {
    let rng = SystemRandom::new();
    let pkcs8_bytes: pkcs8::Document = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let pair: Ed25519KeyPair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();

    let public_key: String = pair.public_key().encode_hex();

    return (pair, public_key)

}