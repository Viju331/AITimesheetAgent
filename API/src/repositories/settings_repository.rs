use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::AppResult;

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

pub struct Setting {
    pub id: String,
    pub key: String,
    pub value: String,
    pub is_encrypted: bool,
    pub updated_at: String,
}

// ---------------------------------------------------------------------------
// Row mapper
// ---------------------------------------------------------------------------

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Setting> {
    Ok(Setting {
        id:           row.get(0)?,
        key:          row.get(1)?,
        value:        row.get(2)?,
        is_encrypted: row.get::<_, i64>(3)? != 0,
        updated_at:   row.get(4)?,
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return every row in the `settings` table.
pub fn get_all(conn: &Connection) -> AppResult<Vec<Setting>> {
    let mut stmt = conn.prepare(
        "SELECT id, key, value, is_encrypted, updated_at FROM settings ORDER BY key",
    )?;

    let rows = stmt.query_map([], map_row)?;

    let mut settings = Vec::new();
    for row in rows {
        settings.push(row?);
    }
    Ok(settings)
}

/// Return a single setting by its key, or `None` if it does not exist.
pub fn get_by_key(conn: &Connection, key: &str) -> AppResult<Option<Setting>> {
    let mut stmt = conn.prepare(
        "SELECT id, key, value, is_encrypted, updated_at FROM settings WHERE key = ?1",
    )?;

    let mut rows = stmt.query_map(params![key], map_row)?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

/// Insert a new setting or update the value/encryption flag if the key already exists.
/// Returns the current (post-upsert) state of the row.
pub fn upsert(
    conn: &Connection,
    key: &str,
    value: &str,
    is_encrypted: bool,
) -> AppResult<Setting> {
    let now = Utc::now().to_rfc3339();
    let is_encrypted_int: i64 = if is_encrypted { 1 } else { 0 };

    // Check whether a row with this key already exists so we can reuse its id.
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(ref id) = existing_id {
        conn.execute(
            "UPDATE settings SET value = ?1, is_encrypted = ?2, updated_at = ?3 WHERE id = ?4",
            params![value, is_encrypted_int, now, id],
        )?;
    } else {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO settings (id, key, value, is_encrypted, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, key, value, is_encrypted_int, now],
        )?;
    }

    // Re-fetch to return the authoritative stored state.
    get_by_key(conn, key)?.ok_or_else(|| {
        crate::errors::AppError::Database(
            format!("setting '{}' not found after upsert", key),
        )
    })
}

/// Delete a setting by key. Does nothing (no error) if the key does not exist.
pub fn delete(conn: &Connection, key: &str) -> AppResult<()> {
    conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
    Ok(())
}
