use hex::ToHex;
use openssl::sha;
use serde::{ Serialize, Deserialize };
// use bls_signatures::{PrivateKey, PublicKey, Serialize as OtherSerialize, Signature};
use ring::{pkcs8, rand::SystemRandom,signature::{KeyPair, Ed25519KeyPair, UnparsedPublicKey}};
use ring::signature;
use serde_json::{to_string, from_str};

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

    pub fn sign_transaction(&mut self, private_key: &Ed25519KeyPair) {
        let tx_json = to_string(&self).expect("Failed to parse transaction");

        let signature_bytes = private_key.sign(tx_json.as_bytes());

        self.signature = signature_bytes.encode_hex();
    }

    pub fn generate_transaction_id(&mut self) -> String {
        let mut hasher = sha::Sha256::new();
        hasher.update(to_string(self).unwrap().as_bytes());
        let digest = hasher.finish().to_vec();
        self.id = hex::encode(&digest);
        return hex::encode(&digest);
    }

    pub fn verify_transaction(&self, public_key_str: String) -> bool{
        let mut temp_tx = self.clone();
        temp_tx.signature = String::new();
        let msg = to_string(&temp_tx).expect("Failed to parse transaction");

        let public_key_bytes = hex::decode(public_key_str).unwrap();
        let pub_slice = public_key_bytes.as_slice();
        let public_key = UnparsedPublicKey::new(&signature::ED25519, pub_slice);

        let verify = public_key.verify(msg.as_bytes(), hex::decode(&self.signature)
        .expect("Failed to parse signature").as_slice());

        match verify {
            Ok(_) => {return true;},
            Err(_) => {return false;}
        }
    }
}

pub fn generate_key_pair() -> (Ed25519KeyPair, String) {
    let rng = SystemRandom::new();
    let pkcs8_bytes: pkcs8::Document = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let pair: Ed25519KeyPair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();

    let public_key: String = pair.public_key().encode_hex();

    return (pair, public_key)

}
