#[cfg(test)]
mod tests {
    use crate::transaction_header::Transaction;
    use rand::{distributions::DistString, thread_rng, Rng};
    use rand::distributions::{Uniform, Alphanumeric};
    use std::time::UNIX_EPOCH;

    #[test]
    fn test_root() {
        let mut rng = thread_rng();
        let dist = Uniform::new(0.0, 1000.0);

        // Creating a pool of random transactions
        let mut transactions = Vec::new();
        for _ in 0..10000 {
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
            let tx = Transaction { id: id_string, from, to, timestamp, amount, fee: 0.01 * amount, signature:  sig};
            transactions.push(tx.clone());
        }
    }
}