use rusqlite::{params, Connection};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

pub struct TimesheetFeedback {
    pub id: String,
    pub timesheet_id: String,
    pub original_text: String,
    pub edited_text: String,
    pub original_verb: Option<String>,
    pub edited_verb: Option<String>,
    pub created_at: String,
}

const COLUMNS: &str = "id, timesheet_id, original_text, edited_text, original_verb, edited_verb, created_at";

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TimesheetFeedback> {
    Ok(TimesheetFeedback {
        id: row.get(0)?,
        timesheet_id: row.get(1)?,
        original_text: row.get(2)?,
        edited_text: row.get(3)?,
        original_verb: row.get(4)?,
        edited_verb: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub fn insert(
    conn: &Connection,
    timesheet_id: &str,
    original_text: &str,
    edited_text: &str,
    original_verb: Option<&str>,
    edited_verb: Option<&str>,
) -> AppResult<TimesheetFeedback> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO timesheet_feedback
            (id, timesheet_id, original_text, edited_text, original_verb, edited_verb, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, timesheet_id, original_text, edited_text, original_verb, edited_verb, created_at],
    )?;

    let sql = format!("SELECT {COLUMNS} FROM timesheet_feedback WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).map_err(AppError::from)
}

/// All recorded feedback, used to rebuild the style profile's learned verb
/// overrides whenever it's refreshed (P5-017).
pub fn get_all(conn: &Connection) -> AppResult<Vec<TimesheetFeedback>> {
    let sql = format!("SELECT {COLUMNS} FROM timesheet_feedback ORDER BY created_at");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    let mut feedback = Vec::new();
    for row in rows {
        feedback.push(row?);
    }
    Ok(feedback)
}
