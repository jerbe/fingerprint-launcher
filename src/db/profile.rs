use rusqlite::{Connection, params};
use crate::models::{Profile, ProfileBrowser};

pub fn insert_profile(conn: &Connection, name: &str) -> rusqlite::Result<i64> {
    conn.execute("INSERT INTO profiles (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_profile(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM profiles WHERE id=?1", params![id])?;
    Ok(())
}

pub fn list_profiles(conn: &Connection, page: usize, page_size: usize) -> rusqlite::Result<Vec<Profile>> {
    let offset = (page.saturating_sub(1)) * page_size;
    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, updated_at FROM profiles ORDER BY id DESC LIMIT ?1 OFFSET ?2"
    )?;
    let rows = stmt.query_map(params![page_size as i64, offset as i64], |row| {
        Ok(Profile {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn count_profiles(conn: &Connection) -> rusqlite::Result<usize> {
    conn.query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get::<_, usize>(0))
}

pub fn profile_name_exists(conn: &Connection, name: &str, exclude_id: Option<i64>) -> bool {
    match exclude_id {
        Some(id) => conn.query_row(
            "SELECT COUNT(*) FROM profiles WHERE name=?1 AND id!=?2",
            params![name, id],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) > 0,
        None => conn.query_row(
            "SELECT COUNT(*) FROM profiles WHERE name=?1",
            params![name],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) > 0,
    }
}

pub fn upsert_profile_browser(conn: &Connection, profile_id: i64, browser_id: i64, launch_args: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM profile_browsers WHERE profile_id=?1 AND browser_id=?2",
        params![profile_id, browser_id],
    )?;
    conn.execute(
        "INSERT INTO profile_browsers (profile_id, browser_id, launch_args) VALUES (?1, ?2, ?3)",
        params![profile_id, browser_id, launch_args],
    )?;
    Ok(())
}

pub fn list_profile_browsers(conn: &Connection, profile_id: i64) -> rusqlite::Result<Vec<ProfileBrowser>> {
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, browser_id, launch_args, created_at, updated_at FROM profile_browsers WHERE profile_id=?1"
    )?;
    let rows = stmt.query_map(params![profile_id], |row| {
        Ok(ProfileBrowser {
            id: row.get(0)?,
            profile_id: row.get(1)?,
            browser_id: row.get(2)?,
            launch_args: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    rows.collect()
}
