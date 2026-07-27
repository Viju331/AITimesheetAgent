use rusqlite::{params, Connection};
use crate::errors::AppResult;
use crate::integrations::models::CommunicationSignal;

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CommunicationSignal> {
    Ok(CommunicationSignal {
        id: row.get(0)?,
        account_id: row.get(1)?,
        source: row.get(2)?,
        signal_type: row.get(3)?,
        content: row.get(4)?,
        ticket_reference: row.get(5)?,
        signal_date: row.get(6)?,
        confidence: row.get(7)?,
        synced_at: row.get(8)?,
    })
}

pub fn get_by_date(conn: &Connection, date: &str) -> AppResult<Vec<CommunicationSignal>> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, source, signal_type, content, ticket_reference,
                signal_date, confidence, synced_at
         FROM communication_signals WHERE signal_date = ?1 ORDER BY confidence DESC",
    )?;
    let rows = stmt.query_map(params![date], map_row)?;
    let mut signals = Vec::new();
    for row in rows { signals.push(row?); }
    Ok(signals)
}

pub fn insert(conn: &Connection, signal: &CommunicationSignal) -> AppResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO communication_signals
            (id, account_id, source, signal_type, content, ticket_reference, signal_date, confidence, synced_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            signal.id, signal.account_id, signal.source, signal.signal_type, signal.content,
            signal.ticket_reference, signal.signal_date, signal.confidence, now
        ],
    )?;
    Ok(())
}
