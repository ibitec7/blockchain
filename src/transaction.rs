use openssl::sha;
use serde::{ Serialize, Deserialize };
use bls_signatures::{PrivateKey, PublicKey, Serialize as OtherSerialize, Signature};
use serde_json::{to_string, from_str};
use rand::thread_rng;

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

impl Transaction {
    pub fn new(from_: String, to_: String, time: u64, amt: f64, fees:f64) -> Self{
        let mut tx = Transaction {
            id: String::new(),
            from: from_,
            to: to_,
            timestamp: time,
            amount: amt,
            fee: fees,
            signature: String::new(),
        };
        tx.generate_transaction_id();

        return tx;

    }

    pub fn serialize(&self) -> String {
        let json_string = to_string(&self).unwrap();
        json_string
    }

    pub fn deserialize(json_string: &str) -> Self {
        let transaction: Transaction = from_str(json_string).expect("Failed to parse transaction");
        transaction
    }

    pub fn sign_transaction(&mut self, private_key: PrivateKey) {
        let tx_json = to_string(&self).expect("Failed to parse transaction");

        let sign = private_key.sign(tx_json);

        self.signature = hex::encode(sign.as_bytes());
    }

    pub fn generate_transaction_id(&mut self) -> String {
        let mut hasher = sha::Sha256::new();
        hasher.update(to_string(self).unwrap().as_bytes());
        let digest = hasher.finish().to_vec();
        self.id = hex::encode(&digest);
        return hex::encode(&digest);
    }

    pub fn verify_transaction(&self, public_key: PublicKey) -> bool{
        let mut temp_tx = self.clone();
        temp_tx.signature = String::new();
        let msg = to_string(&temp_tx).expect("Failed to parse transaction");

        let verify = public_key.verify(Signature::from_bytes(hex::decode(&self.signature).unwrap().as_slice())
            .expect("Failed to parse signature"), msg);

        return verify;
    }
}

pub fn generate_key_pair() -> (PrivateKey, PublicKey) {
    let private_key = PrivateKey::generate(&mut thread_rng());
    let public_key = private_key.public_key();

    return (private_key, public_key)

}
