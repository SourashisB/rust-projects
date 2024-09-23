// src/handlers/regulator.rs
use crate::db::DB;
use crate::models::transaction::Transaction;
use rusqlite::{params, Result};

pub fn view_transactions(db: &DB) -> Result<Vec<Transaction>> {
    let mut stmt = db.conn.prepare("SELECT id, buyer_id, seller_id, amount, asset, timestamp FROM transactions")?;
    let transaction_iter = stmt.query_map([], |row| {
        Ok(Transaction {
            id: row.get(0)?,
            buyer_id: row.get(1)?,
            seller_id: row.get(2)?,
            amount: row.get(3)?,
            asset: row.get(4)?,
            timestamp: row.get(5)?,
        })
    })?;

    let mut transactions = Vec::new();
    for transaction in transaction_iter {
        transactions.push(transaction?);
    }

    Ok(transactions)
}

pub fn view_wallets(db: &DB) -> Result<Vec<(i32, f64)>> {
    let mut stmt = db.conn.prepare("SELECT account_id, balance FROM wallets")?;
    let wallet_iter = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    let mut wallets = Vec::new();
    for wallet in wallet_iter {
        wallets.push(wallet?);
    }

    Ok(wallets)
}

// Add more regulator functionalities