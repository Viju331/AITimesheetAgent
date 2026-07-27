use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

/// One row per commit. Maps onto the pre-existing `git_activities` table
/// (named for the broader concept introduced in Phase 0/1) which already
/// carries exactly the commit-level fields P4-014 calls for.
pub struct GitActivity {
    pub id: String,
    pub project_id: String,
    pub commit_hash: String,
    pub branch: Option<String>,
    pub commit_date: String,
    pub commit_message: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub files_changed: i64,
    pub insertions: i64,
    pub deletions: i64,
    pub change_type: Option<String>,
    pub scanned_at: String,
}

const COLUMNS: &str = "id, project_id, commit_hash, branch, commit_date, commit_message,
                        author_name, author_email, files_changed, insertions, deletions,
                        change_type, scanned_at";

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GitActivity> {
    Ok(GitActivity {
        id: row.get(0)?,
        project_id: row.get(1)?,
        commit_hash: row.get(2)?,
        branch: row.get(3)?,
        commit_date: row.get(4)?,
        commit_message: row.get(5)?,
        author_name: row.get(6)?,
        author_email: row.get(7)?,
        files_changed: row.get(8)?,
        insertions: row.get(9)?,
        deletions: row.get(10)?,
        change_type: row.get(11)?,
        scanned_at: row.get(12)?,
    })
}

pub fn get_by_date(conn: &Connection, date: &str) -> AppResult<Vec<GitActivity>> {
    let sql = format!(
        "SELECT {COLUMNS} FROM git_activities WHERE commit_date LIKE ?1 ORDER BY commit_date"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![format!("{date}%")], map_row)?;
    let mut activities = Vec::new();
    for row in rows {
        activities.push(row?);
    }
    Ok(activities)
}

pub fn get_by_hash(conn: &Connection, project_id: &str, commit_hash: &str) -> AppResult<Option<GitActivity>> {
    let sql = format!("SELECT {COLUMNS} FROM git_activities WHERE project_id = ?1 AND commit_hash = ?2");
    conn.query_row(&sql, params![project_id, commit_hash], map_row).optional().map_err(AppError::from)
}

#[allow(clippy::too_many_arguments)]
pub fn upsert(
    conn: &Connection,
    project_id: &str,
    commit_hash: &str,
    branch: Option<&str>,
    commit_date: &str,
    commit_message: &str,
    author_name: Option<&str>,
    author_email: Option<&str>,
    files_changed: i64,
    insertions: i64,
    deletions: i64,
    change_type: Option<&str>,
) -> AppResult<GitActivity> {
    let now = Utc::now().to_rfc3339();

    let existing_id: Option<String> = conn
        .query_row(
            "SELECT id FROM git_activities WHERE project_id = ?1 AND commit_hash = ?2",
            params![project_id, commit_hash],
            |row| row.get(0),
        )
        .optional()?;

    let id = if let Some(ref eid) = existing_id {
        conn.execute(
            "UPDATE git_activities
             SET branch = ?1, commit_date = ?2, commit_message = ?3, author_name = ?4,
                 author_email = ?5, files_changed = ?6, insertions = ?7, deletions = ?8,
                 change_type = ?9, scanned_at = ?10
             WHERE id = ?11",
            params![
                branch, commit_date, commit_message, author_name, author_email,
                files_changed, insertions, deletions, change_type, now, eid
            ],
        )?;
        eid.clone()
    } else {
        let new_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO git_activities
                (id, project_id, commit_hash, branch, commit_date, commit_message,
                 author_name, author_email, files_changed, insertions, deletions,
                 change_type, scanned_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                new_id, project_id, commit_hash, branch, commit_date, commit_message,
                author_name, author_email, files_changed, insertions, deletions,
                change_type, now
            ],
        )?;
        new_id
    };

    let sql = format!("SELECT {COLUMNS} FROM git_activities WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).map_err(AppError::from)
}
