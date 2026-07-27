use std::collections::HashMap;
use std::path::PathBuf;
use rusqlite::Connection;
use crate::errors::AppResult;
use crate::parsers::{project_mapper, timestamp_engine, tool_registry};
use crate::repositories::{ai_tool_repository, activity_repository, project_repository, session_repository, timeline_repository};
use crate::scanners::session_discovery;
use crate::services::{activity_extractor, timeline_builder};

#[derive(Debug, Default, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub sessions_scanned: i64,
    pub sessions_skipped_unchanged: i64,
    pub sessions_stored: i64,
    pub activities_extracted: i64,
}

/// Scan every enabled AI tool's session folder for today's activity, map
/// sessions to selected projects, extract+classify activities, and persist
/// everything (sessions, activities, timeline events). Re-entrant: unchanged
/// files (by mtime) are skipped; changed files are re-parsed from scratch (P3-018).
pub fn scan_and_store(conn: &Connection) -> AppResult<ScanSummary> {
    let mut summary = ScanSummary::default();

    let tools = ai_tool_repository::get_all(conn)?;
    let selected_projects = project_repository::get_selected(conn)?;
    if selected_projects.is_empty() {
        return Ok(summary);
    }

    let tool_id_by_name: HashMap<String, String> =
        tools.iter().map(|t| (t.name.clone(), t.id.clone())).collect();

    for tool in tools.iter().filter(|t| t.is_enabled) {
        let folder: Option<PathBuf> = tool
            .session_folder
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| tool_registry::default_folder_for(&tool.name));

        let Some(folder) = folder else { continue };

        let discovered = match session_discovery::discover_today_sessions(&tool.name, &folder) {
            Ok(d) => d,
            Err(e) => {
                log::warn!(target: "session_scan", "Failed to scan tool '{}': {}", tool.name, e);
                continue;
            }
        };

        for item in discovered {
            summary.sessions_scanned += 1;

            let Some(project_path) = item.session.project_path.as_deref() else { continue };
            let Some(project) = project_mapper::match_project(project_path, &selected_projects) else { continue };

            let existing = session_repository::get_by_file_path(conn, &item.session.file_path)?;
            if let Some(ref existing_row) = existing {
                if existing_row.file_mtime == item.file_mtime && item.file_mtime.is_some() {
                    summary.sessions_skipped_unchanged += 1;
                    continue;
                }
                // File changed since last parse — clear stale derived data before re-inserting.
                activity_repository::delete_by_source_session(conn, &existing_row.id)?;
                timeline_repository::delete_by_session(conn, &existing_row.id)?;
                session_repository::delete_by_id(conn, &existing_row.id)?;
            }

            let session_date = timestamp_engine::to_local_date(&item.session.started_at)
                .map(timestamp_engine::format_date)
                .unwrap_or_else(timestamp_engine::today_date_string);

            let raw_summary = item
                .session
                .user_messages()
                .map(|m| m.text.as_str())
                .collect::<Vec<_>>()
                .join("\n---\n");

            let tool_id = tool_id_by_name.get(&tool.name).map(String::as_str);

            let stored_session = session_repository::insert(
                conn,
                Some(&project.id),
                tool_id,
                &session_date,
                &item.session.file_path,
                Some(&raw_summary),
                item.session.messages.len() as i64,
                Some(&item.session.started_at),
                Some(&item.session.ended_at),
                item.file_mtime.as_deref(),
            )?;
            summary.sessions_stored += 1;

            for message in item.session.user_messages() {
                let Some(extracted) = activity_extractor::extract_from_text(&message.text, &message.timestamp)
                else {
                    continue;
                };

                let activity_date = timestamp_engine::to_local_date(&extracted.timestamp)
                    .map(timestamp_engine::format_date)
                    .unwrap_or_else(|| session_date.clone());

                let activity = activity_repository::insert(
                    conn,
                    Some(&project.id),
                    &activity_date,
                    &extracted.category,
                    &extracted.title,
                    Some(&extracted.description),
                    Some(&stored_session.id),
                    None,
                    extracted.confidence_score,
                )?;
                summary.activities_extracted += 1;

                let event_time = timeline_builder::local_time_label(&extracted.timestamp);
                timeline_repository::insert(
                    conn,
                    Some(&project.id),
                    Some(&stored_session.id),
                    Some(&activity.id),
                    &activity_date,
                    &event_time,
                    &extracted.title,
                    Some(&extracted.description),
                )?;
            }
        }
    }

    Ok(summary)
}
