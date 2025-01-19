use std::collections::HashMap;
use crate::transaction::UTXO;

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
}

