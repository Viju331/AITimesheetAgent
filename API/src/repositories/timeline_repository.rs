use rusqlite::{params, Connection};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

pub struct TimelineEvent {
    pub id:          String,
    pub project_id:  Option<String>,
    pub session_id:  Option<String>,
    pub activity_id: Option<String>,
    pub event_date:  String,
    pub event_time:  String,
    pub title:       String,
    pub description: Option<String>,
    pub created_at:  String,
}

// ---------------------------------------------------------------------------
// Row mapper
// ---------------------------------------------------------------------------

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimelineEvent> {
    Ok(TimelineEvent {
        id:          row.get(0)?,
        project_id:  row.get(1)?,
        session_id:  row.get(2)?,
        activity_id: row.get(3)?,
        event_date:  row.get(4)?,
        event_time:  row.get(5)?,
        title:       row.get(6)?,
        description: row.get(7)?,
        created_at:  row.get(8)?,
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return all timeline events for a given date, ordered chronologically.
pub fn get_by_date(conn: &Connection, date: &str) -> AppResult<Vec<TimelineEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, session_id, activity_id, event_date, event_time, title, description, created_at
         FROM timeline_events
         WHERE event_date = ?1
         ORDER BY event_time",
    )?;

    let rows = stmt.query_map(params![date], map_row)?;
    let mut events = Vec::new();
    for row in rows {
        events.push(row?);
    }
    Ok(events)
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    project_id:  Option<&str>,
    session_id:  Option<&str>,
    activity_id: Option<&str>,
    event_date:  &str,
    event_time:  &str,
    title:       &str,
    description: Option<&str>,
) -> AppResult<TimelineEvent> {
    let id         = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO timeline_events
            (id, project_id, session_id, activity_id, event_date, event_time, title, description, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, project_id, session_id, activity_id, event_date, event_time, title, description, created_at],
    )?;

    conn.query_row(
        "SELECT id, project_id, session_id, activity_id, event_date, event_time, title, description, created_at
         FROM timeline_events WHERE id = ?1",
        params![id],
        map_row,
    )
    .map_err(AppError::from)
}

/// Delete all timeline events sourced from a given session (P3-018 refresh).
pub fn delete_by_session(conn: &Connection, session_id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM timeline_events WHERE session_id = ?1", params![session_id])?;
    Ok(())
}
