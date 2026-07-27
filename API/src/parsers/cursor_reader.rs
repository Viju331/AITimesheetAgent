use std::fs;
use std::path::{Path, PathBuf};
use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use crate::errors::AppResult;
use super::reader::SessionReader;
use super::session_model::{MessageRole, NormalizedMessage, NormalizedSession};

/// Reads Cursor chat history out of each workspace's `state.vscdb` SQLite file.
pub struct CursorReader;

impl SessionReader for CursorReader {
    fn tool_name(&self) -> &'static str {
        "cursor"
    }

    fn discover(&self, tool_folder: &Path) -> AppResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        if !tool_folder.is_dir() {
            return Ok(files);
        }

        for entry in fs::read_dir(tool_folder)?.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let db_path = dir.join("state.vscdb");
            if db_path.is_file() {
                files.push(db_path);
            }
        }
        Ok(files)
    }

    fn read(&self, source: &Path) -> AppResult<Option<NormalizedSession>> {
        let workspace_path = source
            .parent()
            .map(|d| d.join("workspace.json"))
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .and_then(|v| v.get("folder").and_then(|f| f.as_str()).map(|s| s.to_string()));

        // Read-only open avoids contending with a live Cursor instance holding the file.
        let conn = match Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        let mut stmt = match conn.prepare(
            "SELECT value FROM ItemTable
             WHERE key LIKE '%chat%' OR key LIKE '%aichat%' OR key LIKE '%composer%'",
        ) {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };

        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut messages: Vec<NormalizedMessage> = Vec::new();
        for row in rows.flatten() {
            if let Ok(parsed) = serde_json::from_str::<Value>(&row) {
                collect_bubbles(&parsed, &mut messages);
            }
        }

        if messages.is_empty() {
            return Ok(None);
        }

        messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        let started_at = messages.first().unwrap().timestamp.clone();
        let ended_at = messages.last().unwrap().timestamp.clone();

        Ok(Some(NormalizedSession {
            tool_name: "cursor".to_string(),
            source_session_id: source.to_string_lossy().to_string(),
            file_path: source.to_string_lossy().to_string(),
            project_path: workspace_path,
            started_at,
            ended_at,
            messages,
        }))
    }
}

fn collect_bubbles(value: &Value, out: &mut Vec<NormalizedMessage>) {
    let Some(tabs) = value.get("tabs").and_then(|t| t.as_array()) else {
        return;
    };
    for tab in tabs {
        let Some(bubbles) = tab.get("bubbles").and_then(|b| b.as_array()) else {
            continue;
        };
        for bubble in bubbles {
            let role = match bubble.get("type").and_then(|t| t.as_str()) {
                Some("user") => MessageRole::User,
                Some("ai") => MessageRole::Assistant,
                _ => continue,
            };
            let Some(text) = bubble.get("text").and_then(|t| t.as_str()) else {
                continue;
            };
            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }

            let timestamp_ms = bubble.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0);
            let timestamp = epoch_ms_to_iso(timestamp_ms);
            if timestamp.is_empty() {
                continue;
            }

            out.push(NormalizedMessage { role, text: trimmed.to_string(), timestamp });
        }
    }
}

fn epoch_ms_to_iso(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms).map(|dt| dt.to_rfc3339()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_bubbles_from_known_shape() {
        let value: Value = serde_json::from_str(
            r#"{
                "tabs": [{
                    "bubbles": [
                        { "type": "user", "text": "fix the dropdown", "timestamp": 1718437934000 },
                        { "type": "ai", "text": "Sure thing.", "timestamp": 1718437940000 }
                    ]
                }]
            }"#,
        )
        .unwrap();

        let mut out = Vec::new();
        collect_bubbles(&value, &mut out);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].role, MessageRole::User);
        assert_eq!(out[0].text, "fix the dropdown");
    }

    #[test]
    fn ignores_unknown_shapes() {
        let value: Value = serde_json::from_str(r#"{"unrelated": true}"#).unwrap();
        let mut out = Vec::new();
        collect_bubbles(&value, &mut out);
        assert!(out.is_empty());
    }
}
