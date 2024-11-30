// src/models/wallet.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub id: i32,
    pub account_id: i32,
    pub balance: f64,
}