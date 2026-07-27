use rusqlite::{params, Connection};
use crate::errors::AppResult;
use crate::integrations::models::CalendarEvent;

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CalendarEvent> {
    let attendees_json: Option<String> = row.get(10)?;
    let attendees = attendees_json
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default();
    Ok(CalendarEvent {
        id: row.get(0)?,
        account_id: row.get(1)?,
        provider_event_id: row.get(2)?,
        title: row.get(3)?,
        event_type: row.get(4)?,
        start_time: row.get(5)?,
        end_time: row.get(6)?,
        event_date: row.get(7)?,
        duration_minutes: row.get(8)?,
        is_organizer: row.get::<_, i64>(9)? != 0,
        attendees,
    })
}

pub fn get_by_date(conn: &Connection, date: &str) -> AppResult<Vec<CalendarEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, account_id, provider_event_id, title, event_type, start_time, end_time,
                event_date, duration_minutes, is_organizer, attendees_json
         FROM calendar_events WHERE event_date = ?1 ORDER BY start_time",
    )?;
    let rows = stmt.query_map(params![date], map_row)?;
    let mut events = Vec::new();
    for row in rows { events.push(row?); }
    Ok(events)
}

pub fn upsert(conn: &Connection, event: &CalendarEvent) -> AppResult<()> {
    let attendees_json = serde_json::to_string(&event.attendees).ok();
    let is_organizer_flag: i64 = if event.is_organizer { 1 } else { 0 };

    conn.execute(
        "INSERT INTO calendar_events
            (id, account_id, provider_event_id, title, event_type, start_time, end_time,
             event_date, duration_minutes, is_organizer, attendees_json)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
         ON CONFLICT(account_id, provider_event_id) DO UPDATE SET
         title=excluded.title, event_type=excluded.event_type, start_time=excluded.start_time,
         end_time=excluded.end_time, duration_minutes=excluded.duration_minutes,
         is_organizer=excluded.is_organizer, attendees_json=excluded.attendees_json",
        params![
            event.id, event.account_id, event.provider_event_id, event.title, event.event_type,
            event.start_time, event.end_time, event.event_date, event.duration_minutes,
            is_organizer_flag, attendees_json
        ],
    )?;
    Ok(())
}
