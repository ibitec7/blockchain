
#[cfg(test)]
mod tests {
    use crate::tx_mod::{Transaction, generate_key_pair};
    use crate::tx_mod::TransactionMethods;
use crate::simulate::{User, UserMessage};

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
        
        assert!(!tx.verify_transaction(public_key.clone()), "Unsigned transaction should not verify");

        tx.sign_transaction(&key_pair);
        assert!(!tx.signature.is_empty(), "Transaction should have a signature");
        assert!(tx.verify_transaction(public_key.clone()), "Valid signature should verify");

        tx.amount = 200.0;
        assert!(!tx.verify_transaction(public_key), "Tampered transaction should fail verification");
    }

    #[test]
    fn test_generate_key_pair() {
        let (_key_pair, public_key) = generate_key_pair();
        assert!(!public_key.is_empty(), "Generated public key should not be empty");
    }

    #[test]
    fn test_user_new_and_serialization() {
        let user = User::new();

        assert!(!user.user_id.is_empty(), "User ID should not be empty");
        assert_eq!(user.balance, 42000.0, "User should have the default balance");

        let serialized = user.serialize();
        let deserialized = UserMessage::deserialize(&serialized);
        assert_eq!(user.user_id, deserialized.user_id, "User ID should match after deserialization");
        assert_eq!(user.balance, deserialized.balance, "User balance should match after deserialization");
    }

    #[test]
    fn test_simulate_transaction_validity() {
        let user = User::new();
        let other_user1 = "dummy_key_1".to_string();
        let other_user2 = "dummy_key_2".to_string();
        let user_base = vec![user.user_id.clone(), other_user1.clone(), other_user2.clone()];

        let tx = user.simulate_transaction(user_base.clone());

        assert_eq!(tx.from, user.user_id, "Transaction from field should match user id");

        assert!(user_base.contains(&tx.to), "Transaction to field should be in user base");
        assert_ne!(tx.from, tx.to, "Transaction recipient should not be the same as the sender");

        let expected_fee = 0.01 * tx.amount;
        assert!((tx.fee - expected_fee).abs() < f64::EPSILON, "Transaction fee should be 1% of the amount");
    }
}
