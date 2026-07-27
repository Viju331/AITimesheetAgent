use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

pub struct AiSession {
    pub id:            String,
    pub project_id:    Option<String>,
    pub tool_id:       Option<String>,
    pub session_date:  String,
    pub file_path:     String,
    pub raw_summary:   Option<String>,
    pub message_count: i64,
    pub started_at:    Option<String>,
    pub ended_at:       Option<String>,
    pub file_mtime:     Option<String>,
    pub parsed_at:      String,
}

// ---------------------------------------------------------------------------
// Row mapper
// ---------------------------------------------------------------------------

const COLUMNS: &str = "id, project_id, tool_id, session_date, file_path, raw_summary,
                        message_count, started_at, ended_at, file_mtime, parsed_at";

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AiSession> {
    Ok(AiSession {
        id:            row.get(0)?,
        project_id:    row.get(1)?,
        tool_id:       row.get(2)?,
        session_date:  row.get(3)?,
        file_path:     row.get(4)?,
        raw_summary:   row.get(5)?,
        message_count: row.get(6)?,
        started_at:    row.get(7)?,
        ended_at:      row.get(8)?,
        file_mtime:    row.get(9)?,
        parsed_at:     row.get(10)?,
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return all sessions whose `session_date` matches the given date string
/// (expected format: `YYYY-MM-DD`), ordered by `started_at` ascending.
pub fn get_by_date(conn: &Connection, date: &str) -> AppResult<Vec<AiSession>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM ai_sessions WHERE session_date = ?1 ORDER BY started_at"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![date], map_row)?;

    let mut sessions = Vec::new();
    for row in rows {
        sessions.push(row?);
    }
    Ok(sessions)
}

/// Look up a session by its source file path (the natural unique key for incremental refresh).
pub fn get_by_file_path(conn: &Connection, file_path: &str) -> AppResult<Option<AiSession>> {
    let sql = format!("SELECT {COLUMNS} FROM ai_sessions WHERE file_path = ?1");
    conn.query_row(&sql, params![file_path], map_row).optional().map_err(AppError::from)
}

/// Insert a new AI session row and return it.
#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    project_id:    Option<&str>,
    tool_id:       Option<&str>,
    session_date:  &str,
    file_path:     &str,
    raw_summary:   Option<&str>,
    message_count: i64,
    started_at:    Option<&str>,
    ended_at:      Option<&str>,
    file_mtime:    Option<&str>,
) -> AppResult<AiSession> {
    let id        = Uuid::new_v4().to_string();
    let parsed_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO ai_sessions
            (id, project_id, tool_id, session_date, file_path, raw_summary,
             message_count, started_at, ended_at, file_mtime, parsed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            id, project_id, tool_id, session_date, file_path, raw_summary,
            message_count, started_at, ended_at, file_mtime, parsed_at
        ],
    )?;

    let sql = format!("SELECT {COLUMNS} FROM ai_sessions WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).map_err(AppError::from)
}

/// Delete a session by id (caller is responsible for clearing dependent
/// activities/timeline_events first, since those FKs are `ON DELETE SET NULL`/`CASCADE`
/// rather than a strict cascade chain).
pub fn delete_by_id(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM ai_sessions WHERE id = ?1", params![id])?;
    Ok(())
}

/// Delete all sessions for a given date. Does nothing if none exist.
pub fn delete_by_date(conn: &Connection, date: &str) -> AppResult<()> {
    conn.execute("DELETE FROM ai_sessions WHERE session_date = ?1", params![date])?;
    Ok(())
}
