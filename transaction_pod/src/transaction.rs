use hex::ToHex;
use openssl::sha;
// use bls_signatures::{PrivateKey, PublicKey, Serialize as OtherSerialize, Signature};
use ring::signature::{Ed25519KeyPair, UnparsedPublicKey};
use ring::signature;
use serde_json::{to_string, from_str};

use crate::tx_mod::{TransactionMethods, Transaction};

impl TransactionMethods for Transaction {
    fn new(from_: String, to_: String, time: u64, amt: f64, fees:f64) -> Self{
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

    fn serialize(&self) -> String {
        let json_string = to_string(&self).unwrap();
        json_string
    }

    fn deserialize(json_string: &str) -> Self {
        let transaction: Transaction = from_str(json_string).expect("Failed to parse transaction");
        transaction
    }

    fn sign_transaction(&mut self, private_key: &Ed25519KeyPair) {
        let tx_json = to_string(&self).expect("Failed to parse transaction");

        let signature_bytes = private_key.sign(tx_json.as_bytes());

        self.signature = signature_bytes.encode_hex();
    }

    fn generate_transaction_id(&mut self) -> String {
        let mut hasher = sha::Sha256::new();
        hasher.update(to_string(self).unwrap().as_bytes());
        let digest = hasher.finish().to_vec();
        self.id = hex::encode(&digest);
        return hex::encode(&digest);
    }

    fn verify_transaction(&self, public_key_str: String) -> bool{
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

