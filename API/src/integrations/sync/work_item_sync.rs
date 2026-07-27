use rusqlite::Connection;
use uuid::Uuid;
use chrono::Utc;
use crate::errors::AppResult;
use crate::integrations::models::{SyncResult, WorkItem};
use crate::integrations::providers::{azure_devops::AzureDevOpsClient, jira::JiraClient};
use crate::repositories::{integration_account_repository, work_item_repository};

/// Sync work items for a single integration account by ID. Returns items synced.
pub fn sync_account(conn: &Connection, account_id: &str) -> AppResult<SyncResult> {
    let account = integration_account_repository::get_by_id(conn, account_id)?;
    let account = match account {
        Some(a) if a.is_active() => a,
        Some(_) => return Ok(SyncResult { account_id: account_id.to_string(), provider: "unknown".into(), items_synced: 0, error: Some("account inactive".into()) }),
        None => return Ok(SyncResult { account_id: account_id.to_string(), provider: "unknown".into(), items_synced: 0, error: Some("account not found".into()) }),
    };

    let result = match account.provider.as_str() {
        "jira" => sync_jira(conn, &account),
        "azure_devops" => sync_ado(conn, &account),
        p => Err(crate::errors::AppError::AiProvider(format!("work item sync not supported for provider '{p}'"))),
    };

    match result {
        Ok(count) => {
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

/// Sync all active Jira and ADO accounts and return per-account results.
pub fn sync_all(conn: &Connection) -> AppResult<Vec<SyncResult>> {
    let accounts = integration_account_repository::get_all(conn)?;
    let mut results = Vec::new();

    for account in accounts {
        if !account.is_active() { continue; }
        if account.provider != "jira" && account.provider != "azure_devops" { continue; }
        results.push(sync_account(conn, &account.id)?);
    }

    Ok(results)
}

fn sync_jira(conn: &Connection, account: &crate::integrations::models::IntegrationAccount) -> AppResult<i64> {
    let token = account.access_token.as_deref()
        .ok_or_else(|| crate::errors::AppError::AiProvider("Jira account missing API token".into()))?;
    let base_url = account.base_url.as_deref()
        .ok_or_else(|| crate::errors::AppError::AiProvider("Jira account missing base_url".into()))?;
    let email = &account.display_name;
    let jira_account_id = account.account_id.as_deref().unwrap_or("");

    let client = JiraClient::new(base_url, email, token);
    let items = client.fetch_assigned_issues(jira_account_id)?;
    let count = items.len() as i64;

    for item in items {
        work_item_repository::upsert(conn, &item)?;
    }
    Ok(count)
}

fn sync_ado(conn: &Connection, account: &crate::integrations::models::IntegrationAccount) -> AppResult<i64> {
    let token = account.access_token.as_deref()
        .ok_or_else(|| crate::errors::AppError::AiProvider("ADO account missing PAT".into()))?;
    let org_url = account.base_url.as_deref()
        .ok_or_else(|| crate::errors::AppError::AiProvider("ADO account missing org_url".into()))?;

    let client = AzureDevOpsClient::new(org_url, token);
    let items = client.fetch_assigned_work_items()?;
    let count = items.len() as i64;

    for item in items {
        work_item_repository::upsert(conn, &item)?;
    }
    Ok(count)
}

/// Create a placeholder WorkItem for inserting a manually-discovered item.
#[allow(dead_code)]
pub fn placeholder_item(account_id: &str, provider: &str, provider_item_id: &str, title: &str) -> WorkItem {
    WorkItem {
        id: Uuid::new_v4().to_string(),
        account_id: account_id.to_string(),
        provider: provider.to_string(),
        provider_item_id: provider_item_id.to_string(),
        title: title.to_string(),
        item_type: None,
        status: Some("open".to_string()),
        priority: None,
        project_key: None,
        project_name: None,
        assigned_to: None,
        due_date: None,
        url: None,
        synced_at: Utc::now().to_rfc3339(),
    }
}
