use ring::{pkcs8, rand::{self, SystemRandom}, signature::{self, Ed25519KeyPair, KeyPair}};
use crate::transaction::Output;
use std::collections::HashMap;

// Implementation of the owner

pub struct Owner {
    pub pub_key: String,
    rng: SystemRandom,
    priv_key: Ed25519KeyPair,
    utxos: HashMap<String, Output>
}

impl Owner {
    fn new() -> Self {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes: pkcs8::Document = 
            signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();

        let pair: Ed25519KeyPair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .unwrap();

        let pub_key: String = hex::encode(pair.public_key());

        Owner {
            pub_key,
            rng,
            priv_key: pair,
            utxos: HashMap::new()
        }
    }

    fn sign_utxo(&mut self, message: String, id: String) -> String {
        // sign the new utxo here
        let signature = hex::encode(self.priv_key.sign(&message.into_bytes()));
        
        let pkcs8_bytes: pkcs8::Document = 
            signature::Ed25519KeyPair::generate_pkcs8(&self.rng).unwrap();

        let pair: Ed25519KeyPair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .unwrap();

        let pub_key: String = hex::encode(pair.public_key());

        self.priv_key = pair;
        self.pub_key = pub_key;
        self.utxos.remove(&id);

        signature
    }
}