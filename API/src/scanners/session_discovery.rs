use std::path::Path;
use crate::errors::AppResult;
use crate::parsers::session_model::NormalizedSession;
use crate::parsers::{timestamp_engine, tool_registry};

pub struct DiscoveredSession {
    pub session: NormalizedSession,
    pub file_mtime: Option<String>,
}

/// Discover and parse today's sessions for a single enabled AI tool (P3-003).
/// Sessions with no messages from today's local date are dropped; the
/// returned session's message list and start/end timestamps are narrowed to
/// today's activity only (P3-010).
pub fn discover_today_sessions(tool_name: &str, tool_folder: &Path) -> AppResult<Vec<DiscoveredSession>> {
    let Some(reader) = tool_registry::reader_for(tool_name) else {
        return Ok(vec![]);
    };

    let sources = reader.discover(tool_folder)?;
    let mut results = Vec::new();

    for source in sources {
        let file_mtime = file_mtime_iso(&source);

        let session = match reader.read(&source) {
            Ok(Some(session)) => session,
            Ok(None) => continue,
            Err(e) => {
                log::warn!(target: "session_discovery", "Failed to read session {:?}: {}", source, e);
                continue;
            }
        };

        let todays_messages: Vec<_> =
            session.messages.iter().filter(|m| timestamp_engine::is_today(&m.timestamp)).cloned().collect();

        if todays_messages.is_empty() {
            continue;
        }

        let mut todays_session = session;
        todays_session.started_at = todays_messages.first().unwrap().timestamp.clone();
        todays_session.ended_at = todays_messages.last().unwrap().timestamp.clone();
        todays_session.messages = todays_messages;

        results.push(DiscoveredSession { session: todays_session, file_mtime });
    }

    Ok(results)
}

fn file_mtime_iso(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let datetime: chrono::DateTime<chrono::Utc> = modified.into();
    Some(datetime.to_rfc3339())
}
