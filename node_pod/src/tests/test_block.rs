#[cfg(test)]
mod tests {
    use crate::definitions::transaction_header::Transaction;
    use rand::{distributions::DistString, thread_rng, Rng};
    use crate::definitions::block_header::{Block, BlockMethods, BlockChain, BlockChainMethods};
    use rand::distributions::Alphanumeric;
    use std::time::UNIX_EPOCH;

    fn generate_random_transactions(n: usize) -> Vec<Transaction> {
        let mut rng = thread_rng();
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
            let amount = 526.9;
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
    fn test_block_hash() {
        let transactions = generate_random_transactions(128);
        let mut rng = thread_rng();
        let dummy_hash = (0..64)
            .map(|_| format!("{:x}", rng.gen_range(0..16_u8)))
            .collect();
        let block = Block::new(transactions, dummy_hash, 0);
        let block_clone = block.clone();

        let block_hash = block.hash();
        let block_clone_hash = block_clone.hash();

        assert!(!block_hash.is_empty());
        assert!(block_hash == block_clone_hash);
    }

    #[test]
    fn test_serde_block() {
        let transactions = generate_random_transactions(512);

        let mut rng = thread_rng();
        let dummy_hash = (0..64)
            .map(|_| format!("{:x}", rng.gen_range(0..16_u8)))
            .collect();
        let block = Block::new(transactions.clone(), dummy_hash, 0);

        let block_json = block.serialize_block();

        let block_clone = Block::deserialize_block(&block_json);

        assert!(block.index == block_clone.index);
        assert!(block.hash == block_clone.hash);
        assert!(block.timestamp == block_clone.timestamp);
        assert!(block.merkle_root == block_clone.merkle_root);
        assert!(block.prev_hash == block_clone.prev_hash);
        assert!(block.transactions.len() == block_clone.transactions.len());
        assert!(block.transactions == block_clone.transactions);
    }

    #[test]
    fn test_validation() {
        let transactions = generate_random_transactions(512);

        let mut rng = thread_rng();
        let dummy_hash = (0..64)
            .map(|_| format!("{:x}", rng.gen_range(0..16_u8)))
            .collect();
        let block = Block::new(transactions.clone(), dummy_hash, 0);

        let valid = block.validate(transactions.clone());
        assert!(valid);
    }

    #[test]
    fn test_equality() {
        let transactions = generate_random_transactions(512);

        let mut rng = thread_rng();
        let dummy_hash = (0..64)
            .map(|_| format!("{:x}", rng.gen_range(0..16_u8)))
            .collect();
        let block = Block::new(transactions.clone(), dummy_hash, 0);
        let block_clone = block.clone();

        assert!(block.is_equal(block_clone));
    }

    #[test]
    fn test_new_blockchain() {
        let blockchain = BlockChain::new();

        assert!(blockchain.chain.len() == 1);
        assert !(blockchain.chain[0].index == 0);
        assert!(blockchain.chain[0].prev_hash == hex::encode(vec![]));
        assert!(blockchain.chain[0].transactions.is_empty());
    }
}