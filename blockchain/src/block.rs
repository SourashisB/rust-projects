use serde::{Serialize, Deserialize};
use chrono::Utc;
use sha2::{Sha256, Digest};
use crate::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub proof: u64,
    pub previous_hash: String,
}

impl Block {
    pub fn new(index: u64, transactions: Vec<Transaction>, proof: u64, previous_hash: String) -> Self {
        Block {
            index,
            timestamp: Utc::now().timestamp(),
            transactions,
            proof,
            previous_hash,
        }
    }

    pub fn hash(&self) -> String {
        let encoded = serde_json::to_string(self).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(encoded);
        format!("{:x}", hasher.finalize())
    }
}