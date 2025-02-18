#[cfg(test)]
mod tests {
    use crate::definitions::transaction_header::Transaction;
    use crate::definitions::merkle_header::{MerkleTree, MerkleMethods};
    use rand::{distributions::DistString, thread_rng, Rng};
    use rand::distributions::{Uniform, Alphanumeric};
    use std::time::UNIX_EPOCH;

    fn generate_random_transactions(n: usize) -> Vec<Transaction> {
        let mut rng = thread_rng();
        let dist = Uniform::new(0.0, 1000.0);
        let mut transactions = Vec::new();
        for _ in 0..n {
            let id_string: String = (0..64)
                .map(|_| format!("{:x}", rng.gen_range(0..16_u8)))
                .collect();
            let to = Alphanumeric.sample_string(&mut rng, 64);
            let from = Alphanumeric.sample_string(&mut rng, 64);
            let sig = Alphanumeric.sample_string(&mut rng, 64);
            let timestamp = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let amount = rng.sample(dist);
            let tx = Transaction {
                id: id_string,
                from,
                to,
                timestamp,
                amount,
                fee: 0.01 * amount,
                signature: sig,
            };
            transactions.push(tx);
        }
        transactions
    }

    #[test]
    fn test_root() {
        let transactions = generate_random_transactions(128);

        let root = MerkleTree::generate_root(&transactions);
        assert!(!root.is_empty());
    }

    #[test]
    fn test_new_genesis() {
        let genesis_tree = MerkleTree::new_genesis();

        assert_eq!(genesis_tree.depth, 0);
        assert!(genesis_tree.leaves.is_empty());
    }

    #[test]
    fn test_new() {
        let transactions = generate_random_transactions(128);
        let tree = MerkleTree::new(&transactions);

        assert_eq!(tree.leaves.len(), transactions.len());

        assert!(tree.depth > 0);
    }

    #[test]
    fn test_generate_proof_and_validate() {
        let transactions = generate_random_transactions(128);
        let tree = MerkleTree::new(&transactions);

        let proof = tree.generate_proof(&transactions[0]);

        let generated_root = MerkleTree::generate_root(&transactions);

        let is_valid = MerkleTree::validate_proof(&proof, transactions[0].clone(), &generated_root);
        assert!(is_valid);
    }

    #[test]
    fn test_generate_root_consistency() {
        let transactions1 = generate_random_transactions(128);
        let transactions2 = transactions1.clone();
        let root1 = MerkleTree::generate_root(&transactions1);
        let root2 = MerkleTree::generate_root(&transactions2);
        assert_eq!(root1, root2);
    }
}