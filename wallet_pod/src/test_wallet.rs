use super::*;

use transaction::{UTXO, Script};
use ledger::Ledger;
use wallet::Wallet;
use std::collections::HashMap;
use ring::signature::{Ed25519KeyPair, KeyPair};

#[cfg(test)]
mod tests {
    use crate::transaction::{Transaction,UTXO, Script};
    use crate::ledger::Ledger;
    use crate::wallet::{Wallet, Transact, Info};
    use std::collections::HashMap;
    use ring::signature::{Ed25519KeyPair, KeyPair};

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert!(!wallet.pub_key.is_empty());
        assert!(wallet.utxos.is_empty());
        assert!(!wallet.ip.is_empty());
    }

    #[test]
    fn test_add_and_remove_utxo() {
        let mut wallet = Wallet::new();
        let utxo = UTXO {
            id: "utxo1".to_string(),
            amount: 50,
            to: wallet.pub_key.clone(),
        };

        wallet.add_utxo(utxo.clone());
        assert_eq!(wallet.get_balance(), 50);

        wallet.remove_utxo(&utxo.id);
        assert_eq!(wallet.get_balance(), 0);
    }

    #[test]
    fn test_sign_utxo() {
        let mut wallet = Wallet::new();
        let message = "test message".to_string();
        let id = "utxo1".to_string();

        let signature = wallet.sign_utxo(message.clone(), id.clone());
        assert!(!signature.is_empty());
    }

    #[test]
    fn test_create_script() {
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let pub_keys = vec![wallet1.pub_key.to_string(), wallet2.pub_key.to_string()];
        let signatures = HashMap::new();
        let script = Wallet::create_script(2, 1, pub_keys.clone(), signatures.clone());

        assert_eq!(script.n_keys, 2);
        assert_eq!(script.min_keys, 1);
        assert_eq!(script.pub_keys, pub_keys);
        assert_eq!(script.signatures, signatures);
    }

    #[test]
    fn verify_script() {
        let mut wallet1 = Wallet::new();
        let mut wallet2 = Wallet::new();

        // Add a UTXO to wallet1
        wallet1.add_utxo(UTXO {
            id: "utxo1".to_string(),
            amount: 50,
            to: wallet1.pub_key.clone(),
        });

        // Create a transaction
        let mut test_tx = Transaction::new(
            "tx1".to_string(),
            0,
            50,
            vec!["utxo1".to_string()],
            vec![UTXO {
                id: "utxo2".to_string(),
                amount: 50,
                to: wallet2.pub_key.clone(),
            }],
            Script::new(2, 1, vec![wallet1.pub_key.clone(), wallet2.pub_key.clone()]),
        );

        let tx_message = test_tx.serialize();

        // Get the signatures for the UTXO
        let signature1 = wallet1.sign_utxo(tx_message.clone(), "utxo1".to_string());
        let signature2 = wallet2.sign_utxo(tx_message.clone(), "utxo1".to_string());

        // Create a script with the signatures
        let mut script = Script::new(
            2,
            1,
            vec![wallet1.pub_key.clone(), wallet2.pub_key.clone()],
        );

        script.signatures.insert(wallet1.pub_key.clone(), signature1);
        script.signatures.insert(wallet2.pub_key.clone(), signature2);

        test_tx.owner = script;

        let mut ledger = Ledger::new();
        ledger.add_utxo(UTXO {
            id: "utxo1".to_string(),
            amount: 50,
            to: wallet1.pub_key.clone(),
        });

        let verification = ledger.verify_script(&test_tx.owner, &test_tx);
        assert!(verification);
    }

    #[test]
    fn test_propose_utxo() {
        let wallet = Wallet::new();
        let inputs = vec!["input1".to_string()];
        let outputs = vec![UTXO {
            id: "utxo1".to_string(),
            amount: 50,
            to: wallet.pub_key.clone(),
        }];
        let pub_keys = vec![wallet.pub_key.clone()];

        wallet.propose_utxo(inputs, outputs, 1, 1, pub_keys);
        // This test will not assert anything as it involves network operations.
        // You can add assertions based on your specific requirements.
    }
}