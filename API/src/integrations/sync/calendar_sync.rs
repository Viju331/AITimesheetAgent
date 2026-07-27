use rusqlite::Connection;
use crate::errors::AppResult;
use crate::integrations::models::SyncResult;
use crate::integrations::providers::microsoft_graph::GraphClient;
use crate::repositories::{calendar_event_repository, integration_account_repository};

/// Sync calendar events for the Microsoft account associated with `account_id`.
pub fn sync_account(conn: &Connection, account_id: &str, date: &str) -> AppResult<SyncResult> {
    let account = integration_account_repository::get_by_id(conn, account_id)?;
    let account = match account {
        Some(a) if a.is_active() => a,
        Some(_) => return Ok(SyncResult { account_id: account_id.to_string(), provider: "microsoft".into(), items_synced: 0, error: Some("account inactive".into()) }),
        None => return Ok(SyncResult { account_id: account_id.to_string(), provider: "microsoft".into(), items_synced: 0, error: Some("account not found".into()) }),
    };

    if account.provider != "microsoft" {
        return Ok(SyncResult {
            account_id: account.id,
            provider: account.provider,
            items_synced: 0,
            error: Some("calendar sync only supported for Microsoft accounts".into()),
        });
    }

    let token = account.access_token.as_deref()
        .ok_or_else(|| crate::errors::AppError::AiProvider("Microsoft account missing access token".into()))?;

    let client = GraphClient::new(token);
    let events_result = client.fetch_calendar_events(date);

    match events_result {
        Ok(events) => {
            let count = events.len() as i64;
            for event in events {
                calendar_event_repository::upsert(conn, &event)?;
            }
            let _ = integration_account_repository::update_sync_timestamp(conn, &account.id);
            Ok(SyncResult { account_id: account.id, provider: account.provider, items_synced: count, error: None })
        }
        Err(e) => {
            let msg = e.to_string();
            let _ = integration_account_repository::set_error(conn, &account.id, &msg);
            Ok(SyncResult { account_id: account.id, provider: account.provider, items_synced: 0, error: Some(msg) })
        }
    }
}

/// Sync calendar events for all active Microsoft accounts.
pub fn sync_all(conn: &Connection, date: &str) -> AppResult<Vec<SyncResult>> {
    let accounts = integration_account_repository::get_all(conn)?;
    let mut results = Vec::new();

    for account in accounts {
        if account.is_active() && account.provider == "microsoft" {
            results.push(sync_account(conn, &account.id, date)?);
        }
    }

    Ok(results)
}
