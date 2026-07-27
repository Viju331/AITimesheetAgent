use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};
use crate::models::UserDto;

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<UserDto> {
    Ok(UserDto {
        id: row.get(0)?,
        name: row.get(1)?,
        email: row.get(2)?,
        created_at: row.get(3)?,
    })
}

pub fn get(conn: &Connection) -> AppResult<Option<UserDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, email, created_at FROM users LIMIT 1",
    )?;
    let mut rows = stmt.query_map([], map_row)?;
    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn upsert(conn: &Connection, name: &str, email: Option<&str>) -> AppResult<UserDto> {
    let now = Utc::now().to_rfc3339();

    let existing_id: Option<String> = conn
        .query_row("SELECT id FROM users LIMIT 1", [], |row| row.get(0))
        .optional()?;

    let id = if let Some(ref eid) = existing_id {
        conn.execute(
            "UPDATE users SET name = ?1, email = ?2 WHERE id = ?3",
            params![name, email, eid],
        )?;
        eid.clone()
    } else {
        let new_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO users (id, name, email, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![new_id, name, email, now],
        )?;
        new_id
    };

    conn.query_row(
        "SELECT id, name, email, created_at FROM users WHERE id = ?1",
        params![id],
        map_row,
    )
    .map_err(AppError::from)
}
