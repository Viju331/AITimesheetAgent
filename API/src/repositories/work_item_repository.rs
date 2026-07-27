use rusqlite::{params, Connection};
use crate::errors::AppResult;
use crate::integrations::models::WorkItem;

const COLUMNS: &str = "id, account_id, provider, provider_item_id, title, item_type,
                        status, priority, project_key, project_name, assigned_to,
                        due_date, url, synced_at";

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<WorkItem> {
    Ok(WorkItem {
        id: row.get(0)?,
        account_id: row.get(1)?,
        provider: row.get(2)?,
        provider_item_id: row.get(3)?,
        title: row.get(4)?,
        item_type: row.get(5)?,
        status: row.get(6)?,
        priority: row.get(7)?,
        project_key: row.get(8)?,
        project_name: row.get(9)?,
        assigned_to: row.get(10)?,
        due_date: row.get(11)?,
        url: row.get(12)?,
        synced_at: row.get(13)?,
    })
}

pub fn get_by_account(conn: &Connection, account_id: &str) -> AppResult<Vec<WorkItem>> {
    let sql = format!("SELECT {COLUMNS} FROM work_items WHERE account_id = ?1 ORDER BY title");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![account_id], map_row)?;
    let mut items = Vec::new();
    for row in rows { items.push(row?); }
    Ok(items)
}

pub fn get_all_active(conn: &Connection) -> AppResult<Vec<WorkItem>> {
    let sql = format!("SELECT {COLUMNS} FROM work_items WHERE (status IS NULL OR status != 'closed') ORDER BY provider, project_key, title");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], map_row)?;
    let mut items = Vec::new();
    for row in rows { items.push(row?); }
    Ok(items)
}

pub fn upsert(conn: &Connection, item: &WorkItem) -> AppResult<()> {
    conn.execute(
        "INSERT INTO work_items (id,account_id,provider,provider_item_id,title,item_type,status,priority,project_key,project_name,assigned_to,due_date,url,synced_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
         ON CONFLICT(account_id, provider_item_id) DO UPDATE SET
         title=excluded.title, item_type=excluded.item_type, status=excluded.status,
         priority=excluded.priority, assigned_to=excluded.assigned_to,
         due_date=excluded.due_date, synced_at=excluded.synced_at",
        params![
            item.id, item.account_id, item.provider, item.provider_item_id, item.title,
            item.item_type, item.status, item.priority, item.project_key, item.project_name,
            item.assigned_to, item.due_date, item.url, item.synced_at
        ],
    )?;
    Ok(())
}

pub fn delete_by_account(conn: &Connection, account_id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM work_items WHERE account_id = ?1", params![account_id])?;
    Ok(())
}
