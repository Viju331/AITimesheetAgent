use tauri::State;
use crate::DbState;
use crate::errors::AppResult;
use crate::integrations::{context_engine, oauth_flow};
use crate::integrations::models::{MicrosoftDeviceCodeInfo, SyncResult, WorkContextProfile};
use crate::integrations::providers::{
    azure_devops::AzureDevOpsClient,
    jira::JiraClient,
    slack::SlackClient,
};
use crate::integrations::sync::{calendar_sync, communication_sync, work_item_sync};
use crate::models::{
    CalendarEventDto, CommunicationSignalDto, IntegrationAccountDto, MicrosoftDeviceCodeDto,
    SyncResultDto, WorkContextProfileDto, WorkItemDto,
};
use crate::parsers::timestamp_engine;
use crate::repositories::{
    calendar_event_repository, communication_signal_repository,
    integration_account_repository, work_item_repository,
};

// ── Conversion helpers ──────────────────────────────────────────────────────

fn account_to_dto(a: crate::integrations::models::IntegrationAccount) -> IntegrationAccountDto {
    IntegrationAccountDto {
        id: a.id,
        provider: a.provider,
        display_name: a.display_name,
        account_id: a.account_id,
        status: a.status,
        error_message: a.error_message,
        last_synced_at: a.last_synced_at,
        created_at: a.created_at,
    }
}

fn work_item_to_dto(w: crate::integrations::models::WorkItem) -> WorkItemDto {
    WorkItemDto {
        id: w.id,
        account_id: w.account_id,
        provider: w.provider,
        provider_item_id: w.provider_item_id,
        title: w.title,
        item_type: w.item_type,
        status: w.status,
        priority: w.priority,
        project_key: w.project_key,
        project_name: w.project_name,
        due_date: w.due_date,
        url: w.url,
    }
}

fn calendar_event_to_dto(e: crate::integrations::models::CalendarEvent) -> CalendarEventDto {
    CalendarEventDto {
        id: e.id,
        title: e.title,
        event_type: e.event_type,
        start_time: e.start_time,
        end_time: e.end_time,
        event_date: e.event_date,
        duration_minutes: e.duration_minutes,
        is_organizer: e.is_organizer,
        attendees: e.attendees,
    }
}

fn signal_to_dto(s: crate::integrations::models::CommunicationSignal) -> CommunicationSignalDto {
    CommunicationSignalDto {
        id: s.id,
        source: s.source,
        signal_type: s.signal_type,
        content: s.content,
        ticket_reference: s.ticket_reference,
        signal_date: s.signal_date,
        confidence: s.confidence,
    }
}

fn sync_result_to_dto(r: SyncResult) -> SyncResultDto {
    SyncResultDto { account_id: r.account_id, provider: r.provider, items_synced: r.items_synced, error: r.error }
}

fn context_to_dto(c: WorkContextProfile) -> WorkContextProfileDto {
    WorkContextProfileDto {
        date: c.date,
        active_work_items: c.active_work_items.into_iter().map(work_item_to_dto).collect(),
        calendar_events: c.calendar_events.into_iter().map(calendar_event_to_dto).collect(),
        communication_signals: c.communication_signals.into_iter().map(signal_to_dto).collect(),
        connected_providers: c.connected_providers,
        total_meeting_minutes: c.total_meeting_minutes,
    }
}

// ── Connection commands ─────────────────────────────────────────────────────

/// Connect a Jira account using email + API token (P6-004).
#[tauri::command]
pub fn connect_jira(
    state: State<'_, DbState>,
    base_url: String,
    email: String,
    api_token: String,
) -> AppResult<IntegrationAccountDto> {
    let conn = state.lock_db()?;
    let client = JiraClient::new(&base_url, &email, &api_token);
    let display_name = client.verify_connection()?;
    let jira_account_id = client.get_account_id()?;
    let account = integration_account_repository::upsert(
        &conn,
        "jira",
        &display_name,
        Some(&jira_account_id),
        Some(&base_url),
        Some(&api_token),
        None,
        None,
        None,
    )?;
    Ok(account_to_dto(account))
}

/// Connect an Azure DevOps account using a PAT token (P6-007).
#[tauri::command]
pub fn connect_azure_devops(
    state: State<'_, DbState>,
    org_url: String,
    pat_token: String,
) -> AppResult<IntegrationAccountDto> {
    let conn = state.lock_db()?;
    let client = AzureDevOpsClient::new(&org_url, &pat_token);
    let display_name = client.verify_connection()?;
    let account = integration_account_repository::upsert(
        &conn,
        "azure_devops",
        &display_name,
        None,
        Some(&org_url),
        Some(&pat_token),
        None,
        None,
        None,
    )?;
    Ok(account_to_dto(account))
}

/// Begin Microsoft Device Code OAuth flow (P6-009).
/// Returns the device code info — Angular must show `user_code` + `verification_url` to the user.
#[tauri::command]
pub fn initiate_microsoft_auth(
    tenant_id: String,
    client_id: String,
) -> AppResult<MicrosoftDeviceCodeDto> {
    let info = oauth_flow::initiate_microsoft_device_code(&tenant_id, &client_id)?;
    Ok(MicrosoftDeviceCodeDto {
        device_code: info.device_code,
        user_code: info.user_code,
        verification_url: info.verification_url,
        expires_in: info.expires_in,
        interval: info.interval,
        message: info.message,
    })
}

/// Poll until the user completes the Microsoft Device Code flow, then store the account (P6-009).
#[tauri::command]
pub fn complete_microsoft_auth(
    state: State<'_, DbState>,
    tenant_id: String,
    client_id: String,
    device_code: String,
    interval_secs: i64,
    timeout_secs: i64,
) -> AppResult<IntegrationAccountDto> {
    let conn = state.lock_db()?;
    let info = MicrosoftDeviceCodeInfo {
        device_code,
        user_code: String::new(),
        verification_url: String::new(),
        expires_in: timeout_secs,
        interval: interval_secs,
        message: String::new(),
    };
    let token_response = oauth_flow::poll_microsoft_token(&tenant_id, &client_id, &info, timeout_secs as u64)?;
    let account = integration_account_repository::upsert(
        &conn,
        "microsoft",
        "Microsoft 365",
        None,
        None,
        Some(&token_response.access_token),
        token_response.refresh_token.as_deref(),
        token_response.expires_at.as_deref(),
        token_response.scope.as_deref(),
    )?;
    Ok(account_to_dto(account))
}

/// Connect a Slack workspace using a Bot User OAuth token (P6-011).
#[tauri::command]
pub fn connect_slack(
    state: State<'_, DbState>,
    bot_token: String,
) -> AppResult<IntegrationAccountDto> {
    let conn = state.lock_db()?;
    let client = SlackClient::new(&bot_token);
    let display_name = client.verify_connection()?;
    let account = integration_account_repository::upsert(
        &conn,
        "slack",
        &display_name,
        None,
        None,
        Some(&bot_token),
        None,
        None,
        None,
    )?;
    Ok(account_to_dto(account))
}

/// Disconnect and delete an integration account by ID.
#[tauri::command]
pub fn disconnect_integration(state: State<'_, DbState>, id: String) -> AppResult<()> {
    let conn = state.lock_db()?;
    work_item_repository::delete_by_account(&conn, &id)?;
    integration_account_repository::delete(&conn, &id)
}

// ── Sync commands ───────────────────────────────────────────────────────────

/// Sync work items, calendar events, and communication signals for a single account.
#[tauri::command]
pub fn sync_integration(
    state: State<'_, DbState>,
    id: String,
) -> AppResult<SyncResultDto> {
    let conn = state.lock_db()?;
    let account = integration_account_repository::get_by_id(&conn, &id)?
        .ok_or_else(|| crate::errors::AppError::NotFound(format!("integration account '{id}' not found")))?;

    let result = match account.provider.as_str() {
        "jira" | "azure_devops" => work_item_sync::sync_account(&conn, &id)?,
        "microsoft" => {
            let date = timestamp_engine::today_date_string();
            let _ = calendar_sync::sync_account(&conn, &id, &date);
            let _ = communication_sync::sync_teams_account(&conn, &id, &date);
            work_item_sync::sync_account(&conn, &id).unwrap_or(SyncResult {
                account_id: id.clone(),
                provider: "microsoft".into(),
                items_synced: 0,
                error: None,
            })
        }
        "slack" => {
            let date = timestamp_engine::today_date_string();
            communication_sync::sync_slack_account(&conn, &id, &date)?
        }
        p => SyncResult {
            account_id: id,
            provider: p.to_string(),
            items_synced: 0,
            error: Some(format!("sync not implemented for provider '{p}'")),
        },
    };

    Ok(sync_result_to_dto(result))
}

/// Sync all connected integrations for today.
#[tauri::command]
pub fn sync_all_integrations(state: State<'_, DbState>) -> AppResult<Vec<SyncResultDto>> {
    let conn = state.lock_db()?;
    let date = timestamp_engine::today_date_string();
    let mut results: Vec<SyncResult> = Vec::new();

    results.extend(work_item_sync::sync_all(&conn)?);
    results.extend(calendar_sync::sync_all(&conn, &date)?);
    results.extend(communication_sync::sync_all(&conn, &date)?);

    Ok(results.into_iter().map(sync_result_to_dto).collect())
}

// ── Query commands ──────────────────────────────────────────────────────────

/// List all configured integration accounts.
#[tauri::command]
pub fn get_integrations(state: State<'_, DbState>) -> AppResult<Vec<IntegrationAccountDto>> {
    let conn = state.lock_db()?;
    let accounts = integration_account_repository::get_all(&conn)?;
    Ok(accounts.into_iter().map(account_to_dto).collect())
}

/// Get all active (non-closed) work items across all providers.
#[tauri::command]
pub fn get_work_items(state: State<'_, DbState>) -> AppResult<Vec<WorkItemDto>> {
    let conn = state.lock_db()?;
    let items = work_item_repository::get_all_active(&conn)?;
    Ok(items.into_iter().map(work_item_to_dto).collect())
}

/// Get calendar events for the given date (defaults to today).
#[tauri::command]
pub fn get_calendar_events(
    state: State<'_, DbState>,
    date: Option<String>,
) -> AppResult<Vec<CalendarEventDto>> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);
    let events = calendar_event_repository::get_by_date(&conn, &date)?;
    Ok(events.into_iter().map(calendar_event_to_dto).collect())
}

/// Assemble the full work context profile for the given date (P6-018).
#[tauri::command]
pub fn get_work_context(
    state: State<'_, DbState>,
    date: Option<String>,
) -> AppResult<WorkContextProfileDto> {
    let conn = state.lock_db()?;
    let profile = context_engine::build_for_date(&conn, date.as_deref())?;
    Ok(context_to_dto(profile))
}

/// Get communication signals for the given date.
#[tauri::command]
pub fn get_communication_signals(
    state: State<'_, DbState>,
    date: Option<String>,
) -> AppResult<Vec<CommunicationSignalDto>> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);
    let signals = communication_signal_repository::get_by_date(&conn, &date)?;
    Ok(signals.into_iter().map(signal_to_dto).collect())
}
