// src/models/transaction.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i32,
    pub buyer_id: i32,
    pub seller_id: i32,
    pub amount: f64,
    pub asset: String,
    pub timestamp: String, // ISO 8601 format
}