use rusqlite::{params, Connection};
use crate::errors::{AppError, AppResult};
use crate::models::AiToolDto;

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AiToolDto> {
    Ok(AiToolDto {
        id: row.get(0)?,
        name: row.get(1)?,
        display_name: row.get(2)?,
        session_folder: row.get(3)?,
        is_enabled: row.get::<_, i64>(4)? != 0,
        created_at: row.get(5)?,
    })
}

pub fn get_all(conn: &Connection) -> AppResult<Vec<AiToolDto>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, display_name, session_folder, is_enabled, created_at
         FROM ai_tools ORDER BY display_name",
    )?;
    let rows = stmt.query_map([], map_row)?;
    let mut tools = Vec::new();
    for row in rows {
        tools.push(row?);
    }
    Ok(tools)
}

pub fn update(
    conn: &Connection,
    id: &str,
    is_enabled: bool,
    session_folder: Option<&str>,
) -> AppResult<AiToolDto> {
    let flag = if is_enabled { 1i64 } else { 0i64 };
    let rows_changed = conn.execute(
        "UPDATE ai_tools SET is_enabled = ?1, session_folder = ?2 WHERE id = ?3",
        params![flag, session_folder, id],
    )?;
    if rows_changed == 0 {
        return Err(AppError::NotFound(format!("ai_tool '{}' not found", id)));
    }
    conn.query_row(
        "SELECT id, name, display_name, session_folder, is_enabled, created_at
         FROM ai_tools WHERE id = ?1",
        params![id],
        map_row,
    )
    .map_err(AppError::from)
}
