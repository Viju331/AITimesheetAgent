use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

pub struct Project {
    pub id:           String,
    pub name:         String,
    pub path:         String,
    pub tech_stack:   Option<String>,
    pub has_git:      bool,
    pub git_remote:   Option<String>,
    pub is_selected:  bool,
    pub created_at:   String,
    pub last_scanned: Option<String>,
}

// ---------------------------------------------------------------------------
// Row mapper
// ---------------------------------------------------------------------------

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id:           row.get(0)?,
        name:         row.get(1)?,
        path:         row.get(2)?,
        tech_stack:   row.get(3)?,
        has_git:      row.get::<_, i64>(4)? != 0,
        git_remote:   row.get(5)?,
        is_selected:  row.get::<_, i64>(6)? != 0,
        created_at:   row.get(7)?,
        last_scanned: row.get(8)?,
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return all projects ordered by name.
pub fn get_all(conn: &Connection) -> AppResult<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, tech_stack, has_git, git_remote, is_selected, created_at, last_scanned
         FROM projects
         ORDER BY name",
    )?;

    let rows = stmt.query_map([], map_row)?;

    let mut projects = Vec::new();
    for row in rows {
        projects.push(row?);
    }
    Ok(projects)
}

/// Return only the projects that are currently selected (`is_selected = 1`).
pub fn get_selected(conn: &Connection) -> AppResult<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, tech_stack, has_git, git_remote, is_selected, created_at, last_scanned
         FROM projects
         WHERE is_selected = 1
         ORDER BY name",
    )?;

    let rows = stmt.query_map([], map_row)?;

    let mut projects = Vec::new();
    for row in rows {
        projects.push(row?);
    }
    Ok(projects)
}

/// Insert a project if its `path` is new, or update the mutable columns if it already exists.
/// Returns the current state of the row.
pub fn upsert(
    conn: &Connection,
    name: &str,
    path: &str,
    tech_stack: Option<&str>,
    has_git: bool,
    git_remote: Option<&str>,
) -> AppResult<Project> {
    let now          = Utc::now().to_rfc3339();
    let has_git_int  = if has_git { 1i64 } else { 0i64 };

    // Use the filesystem path as the natural key.
    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM projects WHERE path = ?1",
            params![path],
            |row| row.get(0),
        )
        .optional()?;

    let id: String = if let Some(ref eid) = existing_id {
        conn.execute(
            "UPDATE projects
             SET name = ?1, tech_stack = ?2, has_git = ?3, git_remote = ?4, last_scanned = ?5
             WHERE id = ?6",
            params![name, tech_stack, has_git_int, git_remote, now, eid],
        )?;
        eid.clone()
    } else {
        let new_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO projects (id, name, path, tech_stack, has_git, git_remote, is_selected, created_at, last_scanned)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
            params![new_id, name, path, tech_stack, has_git_int, git_remote, now, now],
        )?;
        new_id
    };

    // Re-fetch the authoritative row.
    conn.query_row(
        "SELECT id, name, path, tech_stack, has_git, git_remote, is_selected, created_at, last_scanned
         FROM projects WHERE id = ?1",
        params![id],
        map_row,
    )
    .map_err(AppError::from)
}

/// Flip the `is_selected` flag for a project by id.
pub fn update_selection(conn: &Connection, id: &str, is_selected: bool) -> AppResult<()> {
    let flag = if is_selected { 1i64 } else { 0i64 };
    let rows_changed = conn.execute(
        "UPDATE projects SET is_selected = ?1 WHERE id = ?2",
        params![flag, id],
    )?;

    if rows_changed == 0 {
        return Err(AppError::NotFound(format!("project '{}' not found", id)));
    }
    Ok(())
}

/// Delete a project by id.  Does nothing (no error) if the id does not exist.
pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}
