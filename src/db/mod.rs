pub mod browser;
pub mod platform;
pub mod account;
pub mod profile;

use rusqlite::Connection;
use std::path::Path;

pub fn init_db(db_path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    create_tables(&conn)?;
    Ok(conn)
}

fn create_tables(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS browsers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            english_name TEXT NOT NULL UNIQUE,
            exe_path TEXT NOT NULL,
            icon_path TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS browser_params (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            browser_id INTEGER NOT NULL REFERENCES browsers(id) ON DELETE CASCADE,
            param_name TEXT NOT NULL,
            value_type TEXT NOT NULL DEFAULT 'text',
            description TEXT NOT NULL DEFAULT ''
        );

        CREATE TABLE IF NOT EXISTS platforms (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            icon TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS profiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            username TEXT NOT NULL,
            password TEXT NOT NULL,
            totp_secret TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );

        CREATE TABLE IF NOT EXISTS account_platforms (
            account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
            platform_id INTEGER NOT NULL REFERENCES platforms(id) ON DELETE CASCADE,
            PRIMARY KEY (account_id, platform_id)
        );

        CREATE TABLE IF NOT EXISTS profile_browsers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            browser_id INTEGER NOT NULL REFERENCES browsers(id) ON DELETE CASCADE,
            launch_args TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL DEFAULT (datetime('now','localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now','localtime'))
        );
        ",
    )?;

    // Migrate: add name column to profiles if missing
    let _: Result<(), _> = conn.execute_batch("ALTER TABLE profiles ADD COLUMN name TEXT");

    Ok(())
}
