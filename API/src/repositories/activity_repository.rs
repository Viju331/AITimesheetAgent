use rusqlite::{params, Connection};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Activity {
    pub id:                String,
    pub project_id:        Option<String>,
    pub activity_date:     String,
    pub activity_type:     String,
    pub title:             String,
    pub description:       Option<String>,
    pub source_session_id: Option<String>,
    pub source_commit_id:  Option<String>,
    pub confidence_score:  f64,
    pub is_flagged:        bool,
    pub created_at:        String,
}

// ---------------------------------------------------------------------------
// Row mapper
// ---------------------------------------------------------------------------

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Activity> {
    Ok(Activity {
        id:                row.get(0)?,
        project_id:        row.get(1)?,
        activity_date:     row.get(2)?,
        activity_type:     row.get(3)?,
        title:             row.get(4)?,
        description:       row.get(5)?,
        source_session_id: row.get(6)?,
        source_commit_id:  row.get(7)?,
        confidence_score:  row.get(8)?,
        is_flagged:        row.get::<_, i64>(9)? != 0,
        created_at:        row.get(10)?,
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Return all activities for the given date (format: `YYYY-MM-DD`),
/// ordered by `created_at` ascending.
pub fn get_by_date(conn: &Connection, date: &str) -> AppResult<Vec<Activity>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, activity_date, activity_type, title, description,
                source_session_id, source_commit_id, confidence_score, is_flagged, created_at
         FROM activities
         WHERE activity_date = ?1
         ORDER BY created_at",
    )?;

    let rows = stmt.query_map(params![date], map_row)?;

    let mut activities = Vec::new();
    for row in rows {
        activities.push(row?);
    }
    Ok(activities)
}

/// Insert a new activity row.  `is_flagged` defaults to `false` on insert;
/// the caller can update it separately if needed.  Returns the inserted row.
#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    project_id:        Option<&str>,
    activity_date:     &str,
    activity_type:     &str,
    title:             &str,
    description:       Option<&str>,
    source_session_id: Option<&str>,
    source_commit_id:  Option<&str>,
    confidence_score:  f64,
) -> AppResult<Activity> {
    let id         = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO activities
             (id, project_id, activity_date, activity_type, title, description,
              source_session_id, source_commit_id, confidence_score, is_flagged, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10)",
        params![
            id,
            project_id,
            activity_date,
            activity_type,
            title,
            description,
            source_session_id,
            source_commit_id,
            confidence_score,
            created_at,
        ],
    )?;

    conn.query_row(
        "SELECT id, project_id, activity_date, activity_type, title, description,
                source_session_id, source_commit_id, confidence_score, is_flagged, created_at
         FROM activities WHERE id = ?1",
        params![id],
        map_row,
    )
    .map_err(AppError::from)
}

/// Delete all activities for a given date.  Does nothing if none exist.
pub fn delete_by_date(conn: &Connection, date: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM activities WHERE activity_date = ?1",
        params![date],
    )?;
    Ok(())
}

/// Delete all activities sourced from a given session.  Used before re-parsing
/// a changed session file so stale activities don't accumulate (P3-018).
pub fn delete_by_source_session(conn: &Connection, session_id: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM activities WHERE source_session_id = ?1",
        params![session_id],
    )?;
    Ok(())
}

/// Attach git corroboration to an existing activity and recompute its
/// confidence score (P4-015 correlation / P4-016 confidence engine).
pub fn update_correlation(
    conn: &Connection,
    id: &str,
    source_commit_id: Option<&str>,
    confidence_score: f64,
) -> AppResult<()> {
    conn.execute(
        "UPDATE activities SET source_commit_id = ?1, confidence_score = ?2 WHERE id = ?3",
        params![source_commit_id, confidence_score, id],
    )?;
    Ok(())
}
