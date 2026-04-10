use rusqlite::{Connection, params};
use crate::models::Account;

pub fn insert_account(conn: &Connection, profile_id: i64, username: &str, password: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO accounts (profile_id, username, password) VALUES (?1, ?2, ?3)",
        params![profile_id, username, password],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_account(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM accounts WHERE id=?1", params![id])?;
    Ok(())
}

pub fn list_accounts_by_profile(conn: &Connection, profile_id: i64) -> rusqlite::Result<Vec<Account>> {
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, username, password, created_at, updated_at FROM accounts WHERE profile_id=?1 ORDER BY id"
    )?;
    let rows = stmt.query_map(params![profile_id], |row| {
        Ok(Account {
            id: row.get(0)?,
            profile_id: row.get(1)?,
            username: row.get(2)?,
            password: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    rows.collect()
}

// Account-Platform many-to-many
pub fn set_account_platforms(conn: &Connection, account_id: i64, platform_ids: &[i64]) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM account_platforms WHERE account_id=?1", params![account_id])?;
    for &pid in platform_ids {
        conn.execute(
            "INSERT INTO account_platforms (account_id, platform_id) VALUES (?1, ?2)",
            params![account_id, pid],
        )?;
    }
    Ok(())
}

pub fn get_account_platform_ids(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT platform_id FROM account_platforms WHERE account_id=?1")?;
    let rows = stmt.query_map(params![account_id], |row| row.get::<_, i64>(0))?;
    rows.collect()
}
