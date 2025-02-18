
use serde::{Serialize, Deserialize};
use ring::signature::UnparsedPublicKey;


#[derive(Serialize, Deserialize, Clone, std::fmt::Debug, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub timestamp: u64,
    pub amount: f64,
    pub fee: f64,
    pub signature: String,
}

pub trait TransactionMethods {
    fn deserialize(json_string: &str) -> Self;

    fn hash_tx(self) -> [u8; 32];

    fn is_equal(self, tx: Transaction) -> bool;

    fn verify_transaction(&self, public_key: UnparsedPublicKey<Vec<u8>>) -> bool;
}
