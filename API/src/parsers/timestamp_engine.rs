use chrono::{DateTime, Local, NaiveDate, Utc};

/// Parse an RFC3339 / ISO 8601 timestamp string into a UTC `DateTime`.
pub fn parse_iso(ts: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts).ok().map(|dt| dt.with_timezone(&Utc))
}

/// Convert an ISO timestamp to the local calendar date the user experienced it on.
pub fn to_local_date(ts: &str) -> Option<NaiveDate> {
    parse_iso(ts).map(|dt| dt.with_timezone(&Local).date_naive())
}

pub fn today_local() -> NaiveDate {
    Local::now().date_naive()
}

/// True when the given timestamp falls on today's local calendar date.
pub fn is_today(ts: &str) -> bool {
    to_local_date(ts).map(|d| d == today_local()).unwrap_or(false)
}

pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn today_date_string() -> String {
    format_date(today_local())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_rfc3339() {
        assert!(parse_iso("2026-06-15T09:32:14.123Z").is_some());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_iso("not-a-date").is_none());
    }

    #[test]
    fn formats_date_as_iso() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        assert_eq!(format_date(date), "2026-06-15");
    }
}
