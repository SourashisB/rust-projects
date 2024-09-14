use sha2::Digest;
use sled::{Db, IVec};
use crate::block::Block;
use crate::transaction::Transaction;
use crate::wallet::Wallet;
use std::collections::HashMap;

pub struct Blockchain {
    db: Db,
    current_transactions: Vec<Transaction>,
    wallets: HashMap<String, f64>,
}

impl Blockchain {
    pub fn new() -> Self {
        let db = sled::open("data/blockchain").unwrap();
        let mut blockchain = Blockchain {
            db,
            current_transactions: Vec::new(),
            wallets: HashMap::new(),
        };

        if blockchain.get_last_block().is_none() {
            blockchain.create_genesis_block();
        }

        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block::new(0, Vec::new(), 100, String::from("0"));
        self.save_block(&genesis_block);
    }

    pub fn create_new_block(&mut self, proof: u64) -> Block {
        let last_block = self.get_last_block().unwrap();
        let new_block = Block::new(
            last_block.index + 1,
            self.current_transactions.clone(),
            proof,
            last_block.hash(),
        );

        self.save_block(&new_block);
        self.current_transactions.clear();
        new_block
    }

    pub fn add_transaction(&mut self, sender: String, recipient: String, amount: f64) -> Result<u64, String> {
        if !self.wallets.contains_key(&sender) {
            return Err(format!("Sender wallet {} does not exist", sender));
        }
        if !self.wallets.contains_key(&recipient) {
            return Err(format!("Recipient wallet {} does not exist", recipient));
        }
        if self.wallets.get(&sender).unwrap() < &amount {
            return Err(format!("Insufficient funds in sender wallet {}", sender));
        }

        *self.wallets.get_mut(&sender).unwrap() -= amount;
        *self.wallets.get_mut(&recipient).unwrap() += amount;

        let transaction = Transaction {
            sender,
            recipient,
            amount,
        };
        self.current_transactions.push(transaction);
        Ok(self.get_last_block().unwrap().index + 1)
    }

    pub fn get_last_block(&self) -> Option<Block> {
        self.db.last()
            .ok()
            .flatten()
            .and_then(|(_, v)| serde_json::from_slice(&v).ok())
    }

    fn save_block(&mut self, block: &Block) {
        let key = block.index.to_be_bytes();
        let value = serde_json::to_vec(block).unwrap();
        self.db.insert(key, value).unwrap();
    }

    pub fn proof_of_work(&self, last_proof: u64) -> u64 {
        let mut proof = 0;
        while !Blockchain::valid_proof(last_proof, proof) {
            proof += 1;
        }
        proof
    }

    fn valid_proof(last_proof: u64, proof: u64) -> bool {
        let guess = format!("{}{}", last_proof, proof);
        let guess_hash = format!("{:x}", sha2::Sha256::digest(guess.as_bytes()));
        guess_hash.starts_with("0000")
    }

    pub fn create_wallet(&mut self) -> Wallet {
        let wallet = Wallet::new();
        self.wallets.insert(wallet.address.clone(), 0.0);
        wallet
    }

    pub fn get_wallet_balance(&self, address: &str) -> Result<f64, String> {
        self.wallets.get(address)
            .cloned()
            .ok_or_else(|| format!("Wallet {} not found", address))
    }

    pub fn mine(&mut self) -> Block {
        let last_block = self.get_last_block().unwrap();
        let last_proof = last_block.proof;
        let proof = self.proof_of_work(last_proof);

        // Reward the miner
        let miner_address = self.create_wallet().address;
        self.add_transaction(String::from("0"), miner_address, 10.0).unwrap();

        self.create_new_block(proof)
    }

    pub fn get_chain(&self) -> Vec<Block> {
        self.db.iter()
            .filter_map(|res| res.ok())
            .filter_map(|(_, v)| serde_json::from_slice(&v).ok())
            .collect()
    }
}