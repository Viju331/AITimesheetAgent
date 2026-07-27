use std::path::PathBuf;
use rusqlite::Connection;
use crate::errors::AppError;

/// Opens (or creates) the SQLite database at the platform app-data path.
pub fn open_connection() -> Result<Connection, AppError> {
    let db_path = db_path()?;

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)?;

    // Enable WAL for concurrent read performance
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;

    log::info!(target: "database", "Opened database at {:?}", db_path);
    Ok(conn)
}

/// Run all pending migration SQL files from the embedded migrations directory.
pub fn run_migrations(conn: &Connection) -> Result<(), AppError> {
    // Ensure the migration tracking table exists
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
        );"
    )?;

    // Embed migration files at compile time
    const MIGRATIONS: &[(&str, &str)] = &[
        ("001_initial_schema", include_str!("../../migrations/001_initial_schema.sql")),
        ("002_session_enhancements", include_str!("../../migrations/002_session_enhancements.sql")),
        ("003_git_intelligence", include_str!("../../migrations/003_git_intelligence.sql")),
        ("004_timesheet_intelligence", include_str!("../../migrations/004_timesheet_intelligence.sql")),
        ("005_integrations", include_str!("../../migrations/005_integrations.sql")),
    ];

    for (name, sql) in MIGRATIONS {
        let already_applied: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )?;

        if already_applied {
            log::debug!(target: "database", "Migration {} already applied, skipping", name);
            continue;
        }

        log::info!(target: "database", "Applying migration: {}", name);
        conn.execute_batch(sql)?;
        conn.execute(
            "INSERT INTO _migrations (name) VALUES (?1)",
            rusqlite::params![name],
        )?;
        log::info!(target: "database", "Migration {} applied successfully", name);
    }

    Ok(())
}

fn db_path() -> Result<PathBuf, AppError> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| AppError::FileSystem("Cannot locate app data directory".into()))?;
    Ok(base.join("ai-timesheet-agent").join("database.db"))
}
