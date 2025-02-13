use hex::ToHex;
use ring::{pkcs8, rand::{self, SystemRandom}, signature::{self, Ed25519KeyPair, KeyPair}};
use crate::transaction::{Transaction, Script, UTXO};
use std::net::UdpSocket;
use openssl::sha::Sha256;
use std::time::SystemTime;
use std::collections::HashMap;
use rdkafka::producer::{BaseProducer, BaseRecord};
use rdkafka::ClientConfig;

// Implementation of the owner's wallet

pub struct Wallet {
    pub pub_key: String,
    rng: SystemRandom,
    priv_key: Ed25519KeyPair,
    pub utxos: HashMap<String, UTXO>,
    pub ip: String,
    pub socket: UdpSocket,
    pub ip_store: HashMap<String, String>, // list of public keys and their IP addresses
    pub producer: BaseProducer
}

pub trait Transact {
    fn create_script(n_keys: u32, min_keys: u32, pub_keys: Vec<String>,
         signatures: HashMap<String, String>) -> Script;

    fn sign_utxo(&mut self, message: String, id: String) -> String;

    fn propose_utxo(&self, inputs: Vec<String>, outputs: Vec<UTXO>, n_keys: u32,
         min_keys: u32, pub_keys: Vec<String>);

    fn listen_transaction(&self) -> impl std::future::Future<Output = String>;
}

pub trait Info {
    fn new() -> Self;

    fn get_balance(&self) -> u64;

    fn add_utxo(&mut self, utxo: UTXO);

    fn remove_utxo(&mut self, id: &String);

    fn reset_key_pair(&mut self);

    fn broadcast_key(&self, pub_key: String);

    fn broadcast_tx(&self, tx: Transaction);
}

impl Transact for Wallet {

    // Listen for a new transaction message (this will always be running as a thread)
    async fn listen_transaction(&self) -> String {
        let mut buffer = [0u8; 4096];

        loop {
            let (amt, _) = self.socket.recv_from(&mut buffer).expect("Failed to receive message");
            let message: String = String::from_utf8(buffer[..amt].to_vec())
                .unwrap_or(String::from("<Invalid UTF-8"));

            return message;
        }
    }

    // Sign a new transaction message once received
    fn sign_utxo(&mut self, message: String, id: String) -> String {
        // sign the new utxo here
        let signature_bytes = self.priv_key.sign(message.as_bytes());
        let signature: String = signature_bytes.encode_hex();

        self.reset_key_pair();

        let _ = self.remove_utxo(&id);

        signature
    }

    // Create a new Script for the transaction
    fn create_script(n_keys: u32, min_keys: u32, pub_keys: Vec<String>,
         signatures: HashMap<String, String>) -> Script {

            Script { n_keys, min_keys, pub_keys, signatures }

    }

    // Propose a new transaction to the peer wallets to sign
    fn propose_utxo(&self, inputs: Vec<String>, outputs: Vec<UTXO>, n_keys: u32,
         min_keys: u32, pub_keys: Vec<String>) {

        let script = Script::new(n_keys, min_keys, pub_keys);
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).
            expect("Failed to create timestamp").as_secs();
        let mut amount: u64 = 0;
        
        for utxo_id in &inputs {
            amount += self.utxos.get(utxo_id).unwrap().amount;
        }
        
        // Hash the transaction

        let mut hasher = Sha256::new();

        hasher.update(&timestamp.to_be_bytes());

        for input in &inputs {
            hasher.update(&input.as_bytes());
        }

        for output in &outputs {
            hasher.update(&output.to_bytes());
        }

        hasher.update(&amount.to_be_bytes());
        hasher.update(&script.to_bytes());

        // Finish the hasher and create a new id for the transaction

        let digest = hasher.finish().to_vec();
        let id = hex::encode(digest);

        let new_tx = Transaction::new(id, timestamp, amount, 
            inputs, outputs.clone(), script);

        for output in &outputs {
            let target_ip = self.ip_store.get(&output.to).unwrap();
            match self.socket.send_to(new_tx.serialize().as_bytes(), target_ip) {
                Ok(_) => {println!("Transaction sent to {}", target_ip);}
                Err(_) => {println!("Failed to send transaction to {}", target_ip);}
            };
        }
    }
}

impl Info for Wallet {

    // Create a new wallet with a new ip, udp socket, keypairs etc.
    fn new() -> Self {
        let ip_base = String::from("127.0.0.1:");
        let mut socket: Option<UdpSocket> = None;
        let mut final_ip = String::new();

        for i in 7000..8000 {
            let ip = format!("{}{}", ip_base, i);
            let temp_socket = UdpSocket::bind(&ip);
            match temp_socket {
                Ok(temp) => {
                    socket = Some(temp);
                    final_ip = ip;
                    break;
                }
                Err(_) => {continue;}
            } 
        }

        let rng = rand::SystemRandom::new();
        let pkcs8_bytes: pkcs8::Document = 
            signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();

        let pair: Ed25519KeyPair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .unwrap();

        let pub_key: String = pair.public_key().encode_hex();

        let producer: BaseProducer = ClientConfig::new()
            .set("bootstrap.servers", "localhost:9092")
            .set("message.timeout.ms", "5000")
            .set("group.id", pub_key.clone())
            .set("enable.auto.commit", "true")
            .set("linger.ms", "5")
            .set("acks", "1")
            .set("compression.type", "lz4")
            .create()
            .expect("Failed to create producer");

        Wallet {
            pub_key,
            rng,
            priv_key: pair,
            utxos: HashMap::new(),
            ip: final_ip,
            socket: socket.unwrap(),
            ip_store: HashMap::new(),
            producer
        }
    }

    fn get_balance(&self) -> u64 {
        self.utxos.values().map(|utxo| utxo.amount).sum()
    }

    fn add_utxo(&mut self, utxo: UTXO) {
        self.utxos.insert(utxo.id.clone(), utxo);
    }

    fn remove_utxo(&mut self, id: &String) {
        self.utxos.remove(id);
    }

    fn reset_key_pair(&mut self) {
        let pkcs8_bytes: pkcs8::Document = 
            signature::Ed25519KeyPair::generate_pkcs8(&self.rng).unwrap();

        let pair: Ed25519KeyPair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .unwrap();

        let pub_key: String = pair.public_key().encode_hex();

        self.priv_key = pair;
        self.pub_key = pub_key;
    }

    fn broadcast_key(&self, pub_key: String) {
        for ip in self.ip_store.values() {
            match self.socket.send_to(pub_key.as_bytes(), ip) {
                Ok(_) => {println!("Public key sent to {}", ip);}
                Err(_) => {println!("Failed to send public key to {}", ip);}
            };
        }
    }

    fn broadcast_tx(&self, tx: Transaction) {
        let tx_string = tx.serialize();
        let record = BaseRecord::to("Transactions")
            .key(&tx.id)
            .payload(&tx_string);

        match self.producer.send(record) {
            Ok(_) => {println!("Transaction sent to Kafka");}
            Err(_) => {println!("Failed to send transaction to Kafka");}
        }
    }
}