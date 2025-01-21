use hex::ToHex;
use ring::{pkcs8, rand::{self, SystemRandom}, signature::{self, Ed25519KeyPair, KeyPair}};
use crate::transaction::{Transaction, Script, UTXO};
use std::net::UdpSocket;
use openssl::sha::{self, Sha256};
use std::time::SystemTime;
use std::collections::HashMap;

// Implementation of the owner

pub struct Wallet {
    pub pub_key: String,
    rng: SystemRandom,
    priv_key: Ed25519KeyPair,
    pub utxos: HashMap<String, UTXO>,
    pub ip: String,
    pub socket: UdpSocket,
    pub ip_store: HashMap<String, String>   // list of public keys and their IP addresses
}

impl Wallet {

    // Create a new wallet with a new ip, udp socket, keypairs etc.
    fn new() -> Self {
        let ip_base = String::from("127.0.0.1:");
        let mut socket: UdpSocket;
        let final_ip: String;

        for i in 7000..8000 {
            let ip = ip_base + &String::from(i);
            let temp_socket = UdpSocket::bind(ip);
            match temp_socket {
                Ok(temp) => {
                    socket = temp;
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

        let pub_key: String = hex::encode(pair.public_key());

        Wallet {
            pub_key,
            rng,
            priv_key: pair,
            utxos: HashMap::new(),
            ip: final_ip,
            socket,
            ip_store: HashMap::new()
        }
    }

    fn listen_transaction(&self) -> String {
        let mut buffer = [0u8; 4096];

        loop {
            let (amt, src) = self.socket.recv_from(&mut buffer);
            let message: String = String::from_utf8(buffer[..amt].to_vec())
                .unwrap_or(String::from("<Invalid UTF-8"));

            return message;
        }
    }

    // Sign a new transaction message once received
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

    // Create a new Script for the transaction
    fn create_script(n_keys: u32, min_keys: u32, pub_keys: Vec<String>,
         signatures: HashMap<String, String>) -> Script {

            Script { n_keys, min_keys, pub_keys, signatures }

         }

    // Propose a new transaction to the peer wallets to sign
    fn propose_utxo(&self, inputs: Vec<String>, outputs: Vec<UTXO>, n_keys: u32,
         min_keys: u32, pub_keys: Vec<String>, socket: UdpSocket) {

        
        let mut buf = [0; 1024];


        let script = Script::new(n_keys, min_keys, pub_keys);
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).
            expect("Failed to create timestamp").as_secs();
        let mut amount: u64 = 0;
        
        for utxo_id in inputs {
            amount += self.utxos.get(&utxo_id).unwrap().amount;
        }
        
        let hasher = Sha256::new();
        hasher.update(&timestamp.swap_bytes());
        hasher.update(&inputs.as_bytes());
        hasher.update(&outputs.as_bytes());
        hasher.update(&amount.swap_bytes());
        hasher.update(&script);

        let digest = hasher.finish().to_vec();
        let id = hex::encode(digest);

        let new_tx = Transaction::new(id, timestamp, amount, 
            inputs, outputs, script);

        for output in outputs {
            let target_ip = self.ip_store.get(&output.to).unwrap();
            self.socket.send_to(transaction.serialize().as_bytes(), target_ip);
        }
    }
}