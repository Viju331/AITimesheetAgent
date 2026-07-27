use rusqlite::Connection;
use crate::errors::AppResult;
use crate::integrations::models::WorkContextProfile;
use crate::parsers::timestamp_engine;
use crate::repositories::{
    calendar_event_repository,
    communication_signal_repository,
    integration_account_repository,
    work_item_repository,
};

/// Assemble the full daily work context from all integration sources (P6-018).
///
/// Loads active work items, calendar events, and communication signals for the
/// given date and packages them into a `WorkContextProfile` that the Angular
/// dashboard and the timesheet pipeline can query.
pub fn build_for_date(conn: &Connection, date: Option<&str>) -> AppResult<WorkContextProfile> {
    let date = date
        .map(|s| s.to_string())
        .unwrap_or_else(timestamp_engine::today_date_string);

    let active_work_items = work_item_repository::get_all_active(conn)?;
    let calendar_events = calendar_event_repository::get_by_date(conn, &date)?;
    let communication_signals = communication_signal_repository::get_by_date(conn, &date)?;

    let all_accounts = integration_account_repository::get_all(conn)?;
    let connected_providers: Vec<String> = all_accounts
        .into_iter()
        .filter(|a| a.is_active())
        .map(|a| a.provider)
        .collect();

    let total_meeting_minutes: i64 = calendar_events
        .iter()
        .filter(|e| e.event_type == "meeting" || e.event_type == "appointment")
        .map(|e| e.duration_minutes)
        .sum();

    Ok(WorkContextProfile {
        date,
        active_work_items,
        calendar_events,
        communication_signals,
        connected_providers,
        total_meeting_minutes,
    })
}
