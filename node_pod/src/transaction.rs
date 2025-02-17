use serde_json::{to_string, from_str};
use ring::signature::UnparsedPublicKey;
use openssl::sha;
use crate::transaction_header::{Transaction, TransactionMethods};

impl TransactionMethods for Transaction {

    fn deserialize(json_string: &str) -> Self {
        let transaction: Transaction = from_str(json_string).expect("Failed to parse transaction");
        transaction
    }

    fn hash_tx(self) -> [u8; 32] {
        let mut hasher = sha::Sha256::new();

        hasher.update(self.id.as_bytes());
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(&self.timestamp.to_be_bytes());
        hasher.update(&self.amount.to_be_bytes());
        hasher.update(&self.fee.to_be_bytes());
        hasher.update(&self.signature.as_bytes());

        hasher.finish()
    }

    fn is_equal(self, tx: Transaction) -> bool {
        let mut predicate: bool = self.id == tx.id;
        predicate = predicate && (self.from == tx.from);
        predicate = predicate && (self.to == tx.to);
        predicate = predicate && (self.timestamp == tx.timestamp);
        predicate = predicate && (self.amount == tx.amount);
        predicate = predicate && (self.fee == tx.fee);
        predicate = predicate && (self.signature == tx.signature);

        predicate
    }

    fn verify_transaction(&self, public_key: UnparsedPublicKey<Vec<u8>>) -> bool{
        let mut temp_tx = self.clone();
        temp_tx.signature = String::new();
        let msg = to_string(&temp_tx).expect("Failed to parse transaction");

        let verify = public_key.verify(msg.as_bytes() ,hex::decode(&self.signature).unwrap().as_slice());

        match verify {
            Ok(_) => true,
            Err(_) => false
        }
    }
}