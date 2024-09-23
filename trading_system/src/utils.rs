// src/utils.rs
use crate::db::DB;
use bcrypt::verify;
use rusqlite::Result;

pub fn authenticate(db: &DB, username: &str, password: &str) -> Result<Option<(i32, String)>> {
    let mut stmt = db.conn.prepare("SELECT id, password, role FROM accounts WHERE username = ?1")?;
    let account = stmt.query_row(params![username], |row| {
        Ok((row.get(0)?, row.get::<_, String>(2)?))
    }).optional()?;

    if let Some((id, hashed_pwd)) = account {
        if verify(password, &hashed_pwd).unwrap_or(false) {
            // Return account id and role
            let role = get_role(db, id)?;
            return Ok(Some((id, role)));
        }
    }

    Ok(None)
}

fn get_role(db: &DB, account_id: i32) -> Result<String> {
    let role: String = db.conn.query_row(
        "SELECT role FROM accounts WHERE id = ?1",
        params![account_id],
        |row| row.get(0),
    )?;
    Ok(role)
}