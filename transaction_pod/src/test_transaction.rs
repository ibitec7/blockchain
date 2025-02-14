#[cfg(test)]
mod tests {
    use crate::transaction::{Transaction, generate_key_pair};

    #[test]
    fn test_transaction_new_and_id_generation() {
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 1620000000, 100.0, 0.1);
        assert!(!tx.id.is_empty(), "Transaction ID should be generated");
    }

    #[test]
    fn test_serialize_deserialize() {
        let tx_original = Transaction::new("Alice".to_string(), "Bob".to_string(), 1620000000, 100.0, 0.1);
        let serialized = tx_original.serialize();
        let tx_deserialized = Transaction::deserialize(&serialized);
        assert_eq!(tx_original.from, tx_deserialized.from);
        assert_eq!(tx_original.to, tx_deserialized.to);
        assert_eq!(tx_original.timestamp, tx_deserialized.timestamp);
        assert_eq!(tx_original.amount, tx_deserialized.amount);
        assert_eq!(tx_original.fee, tx_deserialized.fee);
    }

    #[test]
    fn test_generate_transaction_id() {
        let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 1620000000, 100.0, 0.1);

        let mut tx1 = tx.clone();
        let mut tx2 = tx.clone();

        let id1 = tx1.generate_transaction_id();
        let id2 = tx2.generate_transaction_id();
        
        assert_eq!(id1, id2, "Transaction ID should remain consistent if state doesn't change");
    }

    #[test]
    fn test_sign_and_verify_transaction() {
        let (key_pair, public_key) = generate_key_pair();
        let mut tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 1620000000, 100.0, 0.1);
        
        // Unsigned transaction should fail verification.
        assert!(!tx.verify_transaction(public_key.clone()), "Unsigned transaction should not verify");

        // Sign transaction and verify signature.
        tx.sign_transaction(&key_pair);
        assert!(!tx.signature.is_empty(), "Transaction should have a signature");
        assert!(tx.verify_transaction(public_key.clone()), "Valid signature should verify");

        // Tamper with the transaction and verify failure.
        tx.amount = 200.0;
        assert!(!tx.verify_transaction(public_key), "Tampered transaction should fail verification");
    }

    #[test]
    fn test_generate_key_pair() {
        let (_key_pair, public_key) = generate_key_pair();
        assert!(!public_key.is_empty(), "Generated public key should not be empty");
    }
}
