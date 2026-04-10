use rusqlite::{Connection, params};
use crate::models::Platform;

pub fn insert_platform(conn: &Connection, name: &str, icon: Option<&str>) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO platforms (name, icon) VALUES (?1, ?2)",
        params![name, icon],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_platform(conn: &Connection, id: i64, name: &str, icon: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE platforms SET name=?1, icon=?2 WHERE id=?3",
        params![name, icon, id],
    )?;
    Ok(())
}

pub fn delete_platform(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM platforms WHERE id=?1", params![id])?;
    Ok(())
}

pub fn list_platforms(conn: &Connection) -> rusqlite::Result<Vec<Platform>> {
    let mut stmt = conn.prepare("SELECT id, name, icon, created_at FROM platforms ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Platform {
            id: row.get(0)?,
            name: row.get(1)?,
            icon: row.get(2)?,
            created_at: row.get(3)?,
        })
    })?;
    rows.collect()
}
