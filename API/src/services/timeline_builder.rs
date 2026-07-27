use crate::parsers::timestamp_engine;

pub struct TimelineEntry {
    pub time: String,
    pub title: String,
    pub description: Option<String>,
}

/// Build a chronologically ordered timeline from raw (timestamp, title, description)
/// tuples, formatting each entry's time in the user's local timezone (P3-016).
pub fn build(mut entries: Vec<(String, String, Option<String>)>) -> Vec<TimelineEntry> {
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
        .into_iter()
        .map(|(ts, title, description)| TimelineEntry { time: local_time_label(&ts), title, description })
        .collect()
}

pub fn local_time_label(ts: &str) -> String {
    timestamp_engine::parse_iso(ts)
        .map(|dt| dt.with_timezone(&chrono::Local).format("%H:%M").to_string())
        .unwrap_or_else(|| "00:00".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_entries_chronologically() {
        let entries = vec![
            ("2026-06-15T14:00:00Z".to_string(), "Added Unit Tests".to_string(), None),
            ("2026-06-15T09:00:00Z".to_string(), "Implemented CAMA Neighborhood".to_string(), None),
            ("2026-06-15T11:00:00Z".to_string(), "Fixed Township Dropdown".to_string(), None),
        ];
        let timeline = build(entries);
        assert_eq!(timeline[0].title, "Implemented CAMA Neighborhood");
        assert_eq!(timeline[1].title, "Fixed Township Dropdown");
        assert_eq!(timeline[2].title, "Added Unit Tests");
    }
}
