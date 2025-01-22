use std::collections::HashMap;
use crate::transaction::UTXO;
use crate::transaction::{Transaction, Script};
use hex::FromHex;
use ring::signature;
use ring::signature::{Ed25519KeyPair, Signature, UnparsedPublicKey};

pub struct Ledger {
    utxos: HashMap<String, UTXO>
}

impl Ledger {
    fn new() -> Self {
        Ledger {
            utxos: HashMap::new()
        }
    }

    fn add_utxo(&mut self, utxo: UTXO) {
        self.utxos.insert(utxo.id.clone(), utxo);
    }

    fn spend_utxo(&mut self, utxo_id: &String) -> Option<UTXO> {
        self.utxos.remove(utxo_id)
    }

    fn get_utxo(&self, utxo_id: &String) -> Option<&UTXO> {
        self.utxos.get(utxo_id)
    }

    fn verify_script(&self, script: &Script, tx: &Transaction) -> bool {
        let mut temp_tx = tx.clone();
        temp_tx.owner.signatures = HashMap::new();
        let message_str = temp_tx.serialize();
        let message = message_str.as_bytes();

        if script.n_keys != script.pub_keys.len() as u32{
            return false;
        } else {
            let mut valid_signatures = 0;

            for key in &script.pub_keys {
                let pub_key_bytes = hex::decode(key).unwrap();
                let pub_key = UnparsedPublicKey::new(&signature::ED25519, pub_key_bytes);

                let signature = script.signatures.get(key.as_str());

                match signature {
                    None => {
                        continue;
                    }
                    Some(signature) => {
                        let signature_bytes = hex::decode(signature).unwrap();

                        if pub_key.verify(message, &signature_bytes).is_ok() {
                            valid_signatures += 1;
                        }
                    }
                }
            }
            valid_signatures >= script.min_keys
        }
    }

    fn verify_transaction(&self, tx: &Transaction) -> bool {
        let mut input_sum = 0;
        let mut output_sum = 0;

        for utxo_id in &tx.input {
            match self.get_utxo(utxo_id) {
                Some(utxo) => {
                    input_sum += utxo.amount;
                }
                None => {
                    return false;
                }
            }
        }

        for utxo in &tx.utxo {
            output_sum += utxo.amount;
        }

        input_sum == output_sum && self.verify_script(&tx.owner, &tx)
    }
}

