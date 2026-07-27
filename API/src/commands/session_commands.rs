use tauri::State;
use crate::DbState;
use crate::errors::AppResult;
use crate::models::{ActivityDto, SessionDashboardDto, SessionDto, TimelineEventDto};
use crate::parsers::timestamp_engine;
use crate::repositories::{activity_repository, session_repository, timeline_repository};
use crate::services::session_scan_service::{self, ScanSummary};

fn session_to_dto(s: session_repository::AiSession) -> SessionDto {
    SessionDto {
        id: s.id,
        project_id: s.project_id,
        tool_id: s.tool_id,
        session_date: s.session_date,
        file_path: s.file_path,
        raw_summary: s.raw_summary,
        message_count: s.message_count,
        started_at: s.started_at,
        ended_at: s.ended_at,
        parsed_at: s.parsed_at,
    }
}

fn activity_to_dto(a: activity_repository::Activity) -> ActivityDto {
    ActivityDto {
        id: a.id,
        project_id: a.project_id,
        activity_date: a.activity_date,
        activity_type: a.activity_type,
        title: a.title,
        description: a.description,
        source_session_id: a.source_session_id,
        source_commit_id: a.source_commit_id,
        confidence_score: a.confidence_score,
        is_flagged: a.is_flagged,
        created_at: a.created_at,
    }
}

fn timeline_to_dto(t: timeline_repository::TimelineEvent) -> TimelineEventDto {
    TimelineEventDto {
        id: t.id,
        project_id: t.project_id,
        session_id: t.session_id,
        activity_id: t.activity_id,
        event_date: t.event_date,
        event_time: t.event_time,
        title: t.title,
        description: t.description,
    }
}

/// Scan all enabled AI tools for today's activity and persist sessions/activities/timeline (P3-018 manual refresh entry point).
#[tauri::command]
pub fn scan_sessions(state: State<'_, DbState>) -> AppResult<ScanSummary> {
    let conn = state.lock_db()?;
    session_scan_service::scan_and_store(&conn)
}

#[tauri::command]
pub fn get_sessions(state: State<'_, DbState>, date: Option<String>) -> AppResult<Vec<SessionDto>> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);
    let sessions = session_repository::get_by_date(&conn, &date)?;
    Ok(sessions.into_iter().map(session_to_dto).collect())
}

#[tauri::command]
pub fn get_activities(state: State<'_, DbState>, date: Option<String>) -> AppResult<Vec<ActivityDto>> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);
    let activities = activity_repository::get_by_date(&conn, &date)?;
    Ok(activities.into_iter().map(activity_to_dto).collect())
}

#[tauri::command]
pub fn get_timeline(state: State<'_, DbState>, date: Option<String>) -> AppResult<Vec<TimelineEventDto>> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);
    let events = timeline_repository::get_by_date(&conn, &date)?;
    Ok(events.into_iter().map(timeline_to_dto).collect())
}

#[tauri::command]
pub fn get_session_dashboard(state: State<'_, DbState>, date: Option<String>) -> AppResult<SessionDashboardDto> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);

    let sessions = session_repository::get_by_date(&conn, &date)?;
    let activities = activity_repository::get_by_date(&conn, &date)?;
    let events = timeline_repository::get_by_date(&conn, &date)?;

    let message_count: i64 = sessions.iter().map(|s| s.message_count).sum();

    Ok(SessionDashboardDto {
        session_count: sessions.len() as i64,
        message_count,
        activity_count: activities.len() as i64,
        timeline: events.into_iter().map(timeline_to_dto).collect(),
    })
}
