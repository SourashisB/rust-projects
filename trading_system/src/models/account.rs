// src/models/account.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Customer,
    Regulator,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: i32,
    pub username: String,
    pub password: String, // This should be hashed
    pub role: Role,
}