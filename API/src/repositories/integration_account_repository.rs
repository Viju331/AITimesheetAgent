use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, AppResult};
use crate::integrations::models::IntegrationAccount;

const COLUMNS: &str = "id, provider, display_name, account_id, base_url, access_token,
                        refresh_token, expires_at, scopes, status, error_message,
                        last_synced_at, created_at";

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<IntegrationAccount> {
    Ok(IntegrationAccount {
        id: row.get(0)?,
        provider: row.get(1)?,
        display_name: row.get(2)?,
        account_id: row.get(3)?,
        base_url: row.get(4)?,
        access_token: row.get(5)?,
        refresh_token: row.get(6)?,
        expires_at: row.get(7)?,
        scopes: row.get(8)?,
        status: row.get(9)?,
        error_message: row.get(10)?,
        last_synced_at: row.get(11)?,
        created_at: row.get(12)?,
    })
}

pub fn get_all(conn: &Connection) -> AppResult<Vec<IntegrationAccount>> {
    let sql = format!("SELECT {COLUMNS} FROM integration_accounts ORDER BY provider");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    let mut accounts = Vec::new();
    for row in rows { accounts.push(row?); }
    Ok(accounts)
}

pub fn get_by_provider(conn: &Connection, provider: &str) -> AppResult<Option<IntegrationAccount>> {
    let sql = format!("SELECT {COLUMNS} FROM integration_accounts WHERE provider = ?1 LIMIT 1");
    conn.query_row(&sql, params![provider], map_row).optional().map_err(AppError::from)
}

pub fn get_by_id(conn: &Connection, id: &str) -> AppResult<Option<IntegrationAccount>> {
    let sql = format!("SELECT {COLUMNS} FROM integration_accounts WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).optional().map_err(AppError::from)
}

#[allow(clippy::too_many_arguments)]
pub fn upsert(
    conn: &Connection,
    provider: &str,
    display_name: &str,
    account_id: Option<&str>,
    base_url: Option<&str>,
    access_token: Option<&str>,
    refresh_token: Option<&str>,
    expires_at: Option<&str>,
    scopes: Option<&str>,
) -> AppResult<IntegrationAccount> {
    let now = Utc::now().to_rfc3339();
    let existing_id: Option<String> =
        conn.query_row("SELECT id FROM integration_accounts WHERE provider = ?1", params![provider], |row| row.get(0)).optional()?;

    let id = if let Some(ref eid) = existing_id {
        conn.execute(
            "UPDATE integration_accounts SET display_name=?1, account_id=?2, base_url=?3,
             access_token=?4, refresh_token=?5, expires_at=?6, scopes=?7, status='active', error_message=NULL WHERE id=?8",
            params![display_name, account_id, base_url, access_token, refresh_token, expires_at, scopes, eid],
        )?;
        eid.clone()
    } else {
        let new_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO integration_accounts (id,provider,display_name,account_id,base_url,access_token,refresh_token,expires_at,scopes,status,created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,'active',?10)",
            params![new_id,provider,display_name,account_id,base_url,access_token,refresh_token,expires_at,scopes,now],
        )?;
        new_id
    };

    let sql = format!("SELECT {COLUMNS} FROM integration_accounts WHERE id = ?1");
    conn.query_row(&sql, params![id], map_row).map_err(AppError::from)
}

pub fn update_tokens(conn: &Connection, id: &str, access_token: &str, refresh_token: Option<&str>, expires_at: Option<&str>) -> AppResult<()> {
    conn.execute(
        "UPDATE integration_accounts SET access_token=?1, refresh_token=?2, expires_at=?3, status='active', error_message=NULL WHERE id=?4",
        params![access_token, refresh_token, expires_at, id],
    )?;
    Ok(())
}

pub fn update_sync_timestamp(conn: &Connection, id: &str) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute("UPDATE integration_accounts SET last_synced_at=?1 WHERE id=?2", params![now, id])?;
    Ok(())
}

pub fn set_error(conn: &Connection, id: &str, error: &str) -> AppResult<()> {
    conn.execute("UPDATE integration_accounts SET status='error', error_message=?1 WHERE id=?2", params![error, id])?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM integration_accounts WHERE id = ?1", params![id])?;
    Ok(())
}
