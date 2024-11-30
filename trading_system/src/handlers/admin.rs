// src/handlers/admin.rs
use crate::db::DB;
use crate::models::account::{Account, Role};
use bcrypt::{hash, DEFAULT_COST};
use rusqlite::Result;

pub fn create_user(db: &DB, username: &str, password: &str, role: &str) -> Result<()> {
    let hashed_pwd = hash(password, DEFAULT_COST).unwrap();
    let role_enum = match role.to_lowercase().as_str() {
        "admin" => Role::Admin,
        "customer" => Role::Customer,
        "regulator" => Role::Regulator,
        _ => return Err(rusqlite::Error::InvalidParameterName),
    };

    db.conn.execute(
        "INSERT INTO accounts (username, password, role) VALUES (?1, ?2, ?3)",
        params![username, hashed_pwd, format!("{:?}", role_enum)],
    )?;

    Ok(())
}

// Add more admin functionalities