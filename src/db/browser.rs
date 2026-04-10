use rusqlite::{Connection, params};
use crate::models::{Browser, BrowserParam, ParamValueType};

pub fn insert_browser(conn: &Connection, name: &str, english_name: &str, exe_path: &str, icon_path: Option<&str>) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO browsers (name, english_name, exe_path, icon_path) VALUES (?1, ?2, ?3, ?4)",
        params![name, english_name, exe_path, icon_path],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn english_name_exists(conn: &Connection, english_name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM browsers WHERE english_name=?1",
        params![english_name],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0
}

pub fn english_name_exists_exclude(conn: &Connection, english_name: &str, exclude_id: i64) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM browsers WHERE english_name=?1 AND id!=?2",
        params![english_name, exclude_id],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0
}

pub fn update_browser(conn: &Connection, id: i64, name: &str, english_name: &str, exe_path: &str, icon_path: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE browsers SET name=?1, english_name=?2, exe_path=?3, icon_path=?4, updated_at=datetime('now','localtime') WHERE id=?5",
        params![name, english_name, exe_path, icon_path, id],
    )?;
    Ok(())
}

pub fn delete_browser(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM browsers WHERE id=?1", params![id])?;
    Ok(())
}

pub fn browser_reference_count(conn: &Connection, browser_id: i64) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM profile_browsers WHERE browser_id=?1",
        params![browser_id],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0)
}

pub fn list_browsers(conn: &Connection) -> rusqlite::Result<Vec<Browser>> {
    let mut stmt = conn.prepare("SELECT id, name, english_name, exe_path, icon_path, created_at, updated_at FROM browsers ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok(Browser {
            id: row.get(0)?,
            name: row.get(1)?,
            english_name: row.get(2)?,
            exe_path: row.get(3)?,
            icon_path: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    rows.collect()
}

// Browser params CRUD
pub fn insert_browser_param(conn: &Connection, browser_id: i64, param_name: &str, value_type: &ParamValueType, description: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO browser_params (browser_id, param_name, value_type, description) VALUES (?1, ?2, ?3, ?4)",
        params![browser_id, param_name, value_type.as_str(), description],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_browser_param(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM browser_params WHERE id=?1", params![id])?;
    Ok(())
}

pub fn list_browser_params(conn: &Connection, browser_id: i64) -> rusqlite::Result<Vec<BrowserParam>> {
    let mut stmt = conn.prepare(
        "SELECT id, browser_id, param_name, value_type, description FROM browser_params WHERE browser_id=?1 ORDER BY id"
    )?;
    let rows = stmt.query_map(params![browser_id], |row| {
        let vt: String = row.get(3)?;
        Ok(BrowserParam {
            id: row.get(0)?,
            browser_id: row.get(1)?,
            param_name: row.get(2)?,
            value_type: ParamValueType::from_str(&vt),
            description: row.get(4)?,
        })
    })?;
    rows.collect()
}
