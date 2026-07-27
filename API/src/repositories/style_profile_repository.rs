use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

pub struct StyleProfileRow {
    pub id: String,
    pub user_id: String,
    pub profile_json: String,
    pub source_files: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StyleProfileRow> {
    Ok(StyleProfileRow {
        id: row.get(0)?,
        user_id: row.get(1)?,
        profile_json: row.get(2)?,
        source_files: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

const COLUMNS: &str = "id, user_id, profile_json, source_files, created_at, updated_at";

/// One profile per user — the most recently updated row wins (P5-008).
pub fn get_latest(conn: &Connection) -> AppResult<Option<StyleProfileRow>> {
    let sql = format!("SELECT {COLUMNS} FROM style_profiles ORDER BY updated_at DESC LIMIT 1");
    conn.query_row(&sql, [], map_row).optional().map_err(AppError::from)
}

pub fn upsert(conn: &Connection, user_id: &str, profile_json: &str, source_files: Option<&str>) -> AppResult<StyleProfileRow> {
    let now = Utc::now().to_rfc3339();

    let existing_id: Option<String> =
        conn.query_row("SELECT id FROM style_profiles WHERE user_id = ?1", params![user_id], |row| row.get(0)).optional()?;

    let id = if let Some(ref eid) = existing_id {
        conn.execute(
            "UPDATE style_profiles SET profile_json = ?1, source_files = ?2, updated_at = ?3 WHERE id = ?4",
            params![profile_json, source_files, now, eid],
        )?;
        eid.clone()
    } else {
        let new_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO style_profiles (id, user_id, profile_json, source_files, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![new_id, user_id, profile_json, source_files, now, now],
        )?;
        new_id
    };

    let sql = format!("SELECT {COLUMNS} FROM style_profiles WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).map_err(AppError::from)
}
