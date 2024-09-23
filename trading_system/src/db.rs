// src/db.rs
use rusqlite::{params, Connection, Result};
use std::path::Path;

pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn new(db_path: &str) -> Result<Self> {
        let path = Path::new(db_path);
        let conn = Connection::open(path)?;

        // Initialize tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                role TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS wallets (
                id INTEGER PRIMARY KEY,
                account_id INTEGER,
                balance REAL DEFAULT 0,
                FOREIGN KEY(account_id) REFERENCES accounts(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY,
                buyer_id INTEGER,
                seller_id INTEGER,
                amount REAL,
                asset TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(buyer_id) REFERENCES accounts(id),
                FOREIGN KEY(seller_id) REFERENCES accounts(id)
            )",
            [],
        )?;

        Ok(DB { conn })
    }

    // Add more database interaction functions here
}