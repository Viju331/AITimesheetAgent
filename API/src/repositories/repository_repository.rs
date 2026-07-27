use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

pub struct Repository {
    pub id: String,
    pub project_id: String,
    pub root_path: String,
    pub current_branch: Option<String>,
    pub last_checkout_at: Option<String>,
    pub last_scanned_at: Option<String>,
    pub created_at: String,
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Repository> {
    Ok(Repository {
        id: row.get(0)?,
        project_id: row.get(1)?,
        root_path: row.get(2)?,
        current_branch: row.get(3)?,
        last_checkout_at: row.get(4)?,
        last_scanned_at: row.get(5)?,
        created_at: row.get(6)?,
    })
}

const COLUMNS: &str = "id, project_id, root_path, current_branch, last_checkout_at, last_scanned_at, created_at";

pub fn get_all(conn: &Connection) -> AppResult<Vec<Repository>> {
    let sql = format!("SELECT {COLUMNS} FROM repositories ORDER BY root_path");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    let mut repos = Vec::new();
    for row in rows {
        repos.push(row?);
    }
    Ok(repos)
}

pub fn upsert(
    conn: &Connection,
    project_id: &str,
    root_path: &str,
    current_branch: Option<&str>,
    last_checkout_at: Option<&str>,
) -> AppResult<Repository> {
    let now = Utc::now().to_rfc3339();

    let existing_id: Option<String> = conn
        .query_row("SELECT id FROM repositories WHERE root_path = ?1", params![root_path], |row| row.get(0))
        .optional()?;

    let id = if let Some(ref eid) = existing_id {
        conn.execute(
            "UPDATE repositories
             SET project_id = ?1, current_branch = ?2, last_checkout_at = ?3, last_scanned_at = ?4
             WHERE id = ?5",
            params![project_id, current_branch, last_checkout_at, now, eid],
        )?;
        eid.clone()
    } else {
        let new_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO repositories
                (id, project_id, root_path, current_branch, last_checkout_at, last_scanned_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![new_id, project_id, root_path, current_branch, last_checkout_at, now, now],
        )?;
        new_id
    };

    let sql = format!("SELECT {COLUMNS} FROM repositories WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).map_err(AppError::from)
}
