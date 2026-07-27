use rusqlite::{params, Connection};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

pub struct TaskGroup {
    pub id: String,
    pub project_id: Option<String>,
    pub group_date: String,
    pub title: String,
    pub task_key: String,
    pub ticket_reference: Option<String>,
    pub created_at: String,
}

const COLUMNS: &str = "id, project_id, group_date, title, task_key, ticket_reference, created_at";

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TaskGroup> {
    Ok(TaskGroup {
        id: row.get(0)?,
        project_id: row.get(1)?,
        group_date: row.get(2)?,
        title: row.get(3)?,
        task_key: row.get(4)?,
        ticket_reference: row.get(5)?,
        created_at: row.get(6)?,
    })
}

pub fn get_by_date(conn: &Connection, date: &str) -> AppResult<Vec<TaskGroup>> {
    let sql = format!("SELECT {COLUMNS} FROM task_groups WHERE group_date = ?1 ORDER BY title");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![date], map_row)?;
    let mut groups = Vec::new();
    for row in rows {
        groups.push(row?);
    }
    Ok(groups)
}

pub fn insert(
    conn: &Connection,
    project_id: Option<&str>,
    group_date: &str,
    title: &str,
    task_key: &str,
    ticket_reference: Option<&str>,
) -> AppResult<TaskGroup> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO task_groups (id, project_id, group_date, title, task_key, ticket_reference, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, project_id, group_date, title, task_key, ticket_reference, created_at],
    )?;

    let sql = format!("SELECT {COLUMNS} FROM task_groups WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).map_err(AppError::from)
}

/// Clear today's task groups before regenerating the draft (P5-015) — task
/// groups are a derived view recomputed from current activities, not a
/// hand-maintained record, so a clean replace is correct.
pub fn delete_by_date(conn: &Connection, date: &str) -> AppResult<()> {
    conn.execute("DELETE FROM task_groups WHERE group_date = ?1", params![date])?;
    Ok(())
}
