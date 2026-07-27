use rusqlite::Connection;
use crate::errors::AppResult;
use crate::integrations::models::SyncResult;
use crate::integrations::providers::{microsoft_graph::GraphClient, slack::SlackClient};
use crate::repositories::{communication_signal_repository, integration_account_repository};

/// Sync Slack communication signals for `account_id` on `date`.
pub fn sync_slack_account(conn: &Connection, account_id: &str, date: &str) -> AppResult<SyncResult> {
    let account = integration_account_repository::get_by_id(conn, account_id)?;
    let account = match account {
        Some(a) if a.is_active() => a,
        Some(_) => return Ok(SyncResult { account_id: account_id.to_string(), provider: "slack".into(), items_synced: 0, error: Some("account inactive".into()) }),
        None => return Ok(SyncResult { account_id: account_id.to_string(), provider: "slack".into(), items_synced: 0, error: Some("account not found".into()) }),
    };

    let token = account.access_token.as_deref()
        .ok_or_else(|| crate::errors::AppError::AiProvider("Slack account missing bot token".into()))?;

    let client = SlackClient::new(token);
    match client.fetch_work_signals(date) {
        Ok(signals) => {
            let count = signals.len() as i64;
            for signal in signals {
                let _ = communication_signal_repository::insert(conn, &signal);
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

/// Sync Teams communication signals for `account_id` on `date`.
pub fn sync_teams_account(conn: &Connection, account_id: &str, date: &str) -> AppResult<SyncResult> {
    let account = integration_account_repository::get_by_id(conn, account_id)?;
    let account = match account {
        Some(a) if a.is_active() => a,
        Some(_) => return Ok(SyncResult { account_id: account_id.to_string(), provider: "microsoft".into(), items_synced: 0, error: Some("account inactive".into()) }),
        None => return Ok(SyncResult { account_id: account_id.to_string(), provider: "microsoft".into(), items_synced: 0, error: Some("account not found".into()) }),
    };

    let token = account.access_token.as_deref()
        .ok_or_else(|| crate::errors::AppError::AiProvider("Microsoft account missing access token".into()))?;

    let client = GraphClient::new(token);
    match client.fetch_teams_signals(date) {
        Ok(signals) => {
            let count = signals.len() as i64;
            for signal in signals {
                let _ = communication_signal_repository::insert(conn, &signal);
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

/// Sync communication signals for all active accounts (Slack + Teams).
pub fn sync_all(conn: &Connection, date: &str) -> AppResult<Vec<SyncResult>> {
    let accounts = integration_account_repository::get_all(conn)?;
    let mut results = Vec::new();

    for account in accounts {
        if !account.is_active() { continue; }
        match account.provider.as_str() {
            "slack" => results.push(sync_slack_account(conn, &account.id, date)?),
            "microsoft" => results.push(sync_teams_account(conn, &account.id, date)?),
            _ => {}
        }
    }

    Ok(results)
}
