use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

pub struct GeneratedTimesheet {
    pub id: String,
    pub user_id: Option<String>,
    pub generated_date: String,
    pub content: String,
    pub raw_draft: Option<String>,
    pub status: String,
    pub activity_ids: Option<String>,
    pub ai_provider: Option<String>,
    pub ai_model: Option<String>,
    pub created_at: String,
    pub exported_at: Option<String>,
}

const COLUMNS: &str = "id, user_id, generated_date, content, raw_draft, status, activity_ids,
                        ai_provider, ai_model, created_at, exported_at";

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GeneratedTimesheet> {
    Ok(GeneratedTimesheet {
        id: row.get(0)?,
        user_id: row.get(1)?,
        generated_date: row.get(2)?,
        content: row.get(3)?,
        raw_draft: row.get(4)?,
        status: row.get(5)?,
        activity_ids: row.get(6)?,
        ai_provider: row.get(7)?,
        ai_model: row.get(8)?,
        created_at: row.get(9)?,
        exported_at: row.get(10)?,
    })
}

pub fn get_by_date(conn: &Connection, date: &str) -> AppResult<Vec<GeneratedTimesheet>> {
    let sql = format!("SELECT {COLUMNS} FROM generated_timesheets WHERE generated_date = ?1 ORDER BY created_at DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![date], map_row)?;
    let mut sheets = Vec::new();
    for row in rows {
        sheets.push(row?);
    }
    Ok(sheets)
}

pub fn get_by_id(conn: &Connection, id: &str) -> AppResult<Option<GeneratedTimesheet>> {
    let sql = format!("SELECT {COLUMNS} FROM generated_timesheets WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).optional().map_err(AppError::from)
}

/// Most recent timesheets across all dates, for the history view (P5-014).
pub fn get_history(conn: &Connection, limit: i64) -> AppResult<Vec<GeneratedTimesheet>> {
    let sql = format!("SELECT {COLUMNS} FROM generated_timesheets ORDER BY created_at DESC LIMIT ?1");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![limit], map_row)?;
    let mut sheets = Vec::new();
    for row in rows {
        sheets.push(row?);
    }
    Ok(sheets)
}

pub fn insert(
    conn: &Connection,
    user_id: Option<&str>,
    generated_date: &str,
    content: &str,
    raw_draft: &str,
    activity_ids: &str,
) -> AppResult<GeneratedTimesheet> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO generated_timesheets
            (id, user_id, generated_date, content, raw_draft, status, activity_ids, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'draft', ?6, ?7)",
        params![id, user_id, generated_date, content, raw_draft, activity_ids, created_at],
    )?;

    let sql = format!("SELECT {COLUMNS} FROM generated_timesheets WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).map_err(AppError::from)
}

/// Persist a manual edit to the structured draft content (P5-012).
pub fn update_content(conn: &Connection, id: &str, content: &str) -> AppResult<()> {
    let rows = conn.execute("UPDATE generated_timesheets SET content = ?1 WHERE id = ?2", params![content, id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("timesheet '{id}' not found")));
    }
    Ok(())
}

pub fn update_status(conn: &Connection, id: &str, status: &str) -> AppResult<()> {
    let rows = conn.execute("UPDATE generated_timesheets SET status = ?1 WHERE id = ?2", params![status, id])?;
    if rows == 0 {
        return Err(AppError::NotFound(format!("timesheet '{id}' not found")));
    }
    Ok(())
}
