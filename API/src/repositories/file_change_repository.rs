use rusqlite::{params, Connection};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

pub struct FileChange {
    pub id: String,
    pub commit_id: Option<String>,
    pub repository_id: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub extension: Option<String>,
    pub folder: Option<String>,
    pub change_type: String,
    pub category: String,
    pub insertions: i64,
    pub deletions: i64,
    pub is_working_tree: bool,
    pub detected_at: String,
}

const COLUMNS: &str = "id, commit_id, repository_id, file_path, file_name, extension, folder,
                        change_type, category, insertions, deletions, is_working_tree, detected_at";

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<FileChange> {
    Ok(FileChange {
        id: row.get(0)?,
        commit_id: row.get(1)?,
        repository_id: row.get(2)?,
        file_path: row.get(3)?,
        file_name: row.get(4)?,
        extension: row.get(5)?,
        folder: row.get(6)?,
        change_type: row.get(7)?,
        category: row.get(8)?,
        insertions: row.get(9)?,
        deletions: row.get(10)?,
        is_working_tree: row.get::<_, i64>(11)? != 0,
        detected_at: row.get(12)?,
    })
}

pub fn get_by_commit(conn: &Connection, commit_id: &str) -> AppResult<Vec<FileChange>> {
    let sql = format!("SELECT {COLUMNS} FROM file_changes WHERE commit_id = ?1 ORDER BY file_path");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![commit_id], map_row)?;
    let mut changes = Vec::new();
    for row in rows {
        changes.push(row?);
    }
    Ok(changes)
}

pub fn get_by_repository(conn: &Connection, repository_id: &str, working_tree_only: bool) -> AppResult<Vec<FileChange>> {
    let sql = if working_tree_only {
        format!("SELECT {COLUMNS} FROM file_changes WHERE repository_id = ?1 AND is_working_tree = 1 ORDER BY file_path")
    } else {
        format!("SELECT {COLUMNS} FROM file_changes WHERE repository_id = ?1 ORDER BY file_path")
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![repository_id], map_row)?;
    let mut changes = Vec::new();
    for row in rows {
        changes.push(row?);
    }
    Ok(changes)
}

/// Replace all working-tree (uncommitted) file changes recorded for a
/// repository with a fresh snapshot — there's no stable identity for an
/// uncommitted change to upsert against, so each scan simply replaces the prior set.
pub fn replace_working_tree(conn: &Connection, repository_id: &str, changes: &[NewFileChange]) -> AppResult<()> {
    conn.execute(
        "DELETE FROM file_changes WHERE repository_id = ?1 AND is_working_tree = 1",
        params![repository_id],
    )?;
    for change in changes {
        insert(conn, None, Some(repository_id), change, true)?;
    }
    Ok(())
}

pub struct NewFileChange {
    pub file_path: String,
    pub extension: Option<String>,
    pub folder: Option<String>,
    pub change_type: String,
    pub category: String,
    pub insertions: i64,
    pub deletions: i64,
}

pub fn insert(
    conn: &Connection,
    commit_id: Option<&str>,
    repository_id: Option<&str>,
    change: &NewFileChange,
    is_working_tree: bool,
) -> AppResult<FileChange> {
    let id = Uuid::new_v4().to_string();
    let detected_at = Utc::now().to_rfc3339();
    let file_name = change.file_path.rsplit(['/', '\\']).next().unwrap_or(&change.file_path);
    let working_tree_flag = if is_working_tree { 1i64 } else { 0i64 };

    conn.execute(
        "INSERT INTO file_changes
            (id, commit_id, repository_id, file_path, file_name, extension, folder,
             change_type, category, insertions, deletions, is_working_tree, detected_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            id, commit_id, repository_id, change.file_path, file_name, change.extension, change.folder,
            change.change_type, change.category, change.insertions, change.deletions, working_tree_flag,
            detected_at
        ],
    )?;

    let sql = format!("SELECT {COLUMNS} FROM file_changes WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).map_err(AppError::from)
}

/// Delete all file changes tied to a commit (used before re-scanning a commit, e.g. amended history).
pub fn delete_by_commit(conn: &Connection, commit_id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM file_changes WHERE commit_id = ?1", params![commit_id])?;
    Ok(())
}
