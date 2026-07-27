use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::errors::AppResult;
use super::message_parser::extract_text;
use super::reader::SessionReader;
use super::session_model::{MessageRole, NormalizedMessage, NormalizedSession};

const CANDIDATE_SUBDIRS: &[&str] = &["history", "sessions", "logs"];

/// Reads OpenAI Codex CLI session files. Format is unverified per
/// `docs/research/codex-sessions.md`; this reader fails soft (returns an
/// empty list / `None`) rather than erroring when the expected shape isn't found.
pub struct CodexReader;

impl SessionReader for CodexReader {
    fn tool_name(&self) -> &'static str {
        "codex"
    }

    fn discover(&self, tool_folder: &Path) -> AppResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        if !tool_folder.is_dir() {
            log::warn!(target: "codex_reader", "Codex folder not found at {:?} — skipping", tool_folder);
            return Ok(files);
        }

        let mut search_dirs: Vec<PathBuf> = CANDIDATE_SUBDIRS
            .iter()
            .map(|d| tool_folder.join(d))
            .filter(|d| d.is_dir())
            .collect();

        if search_dirs.is_empty() {
            search_dirs.push(tool_folder.to_path_buf());
        }

        for dir in search_dirs {
            for entry in fs::read_dir(&dir)?.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    files.push(path);
                }
            }
        }
        Ok(files)
    }

    fn read(&self, source: &Path) -> AppResult<Option<NormalizedSession>> {
        let Ok(content) = fs::read_to_string(source) else {
            return Ok(None);
        };
        Ok(parse_session(&content, source))
    }
}

fn parse_session(content: &str, source: &Path) -> Option<NormalizedSession> {
    let parsed: Value = serde_json::from_str(content).ok()?;

    let cwd = parsed.get("cwd").and_then(|c| c.as_str()).map(|s| s.to_string());
    let session_id = parsed.get("id").and_then(|i| i.as_str()).map(|s| s.to_string());
    let created_at = parsed.get("created_at").and_then(|t| t.as_str());

    let mut messages: Vec<NormalizedMessage> = Vec::new();
    if let Some(items) = parsed.get("messages").and_then(|m| m.as_array()) {
        for item in items {
            let role = match item.get("role").and_then(|r| r.as_str()) {
                Some("user") => MessageRole::User,
                Some("assistant") => MessageRole::Assistant,
                _ => continue,
            };
            let Some(content_val) = item.get("content") else {
                continue;
            };
            let Some(text) = extract_text(content_val) else {
                continue;
            };
            let timestamp = item
                .get("timestamp")
                .and_then(|t| t.as_str())
                .or(created_at)
                .unwrap_or_default()
                .to_string();
            if timestamp.is_empty() {
                continue;
            }
            messages.push(NormalizedMessage { role, text, timestamp });
        }
    }

    if messages.is_empty() {
        return None;
    }

    messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let started_at = messages.first().unwrap().timestamp.clone();
    let ended_at = messages.last().unwrap().timestamp.clone();

    Some(NormalizedSession {
        tool_name: "codex".to_string(),
        source_session_id: session_id.unwrap_or_else(|| file_stem(source)),
        file_path: source.to_string_lossy().to_string(),
        project_path: cwd,
        started_at,
        ended_at,
        messages,
    })
}

fn file_stem(path: &Path) -> String {
    path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_expected_shape() {
        let json = r#"{
            "id": "session-1",
            "created_at": "2026-06-15T09:00:00Z",
            "cwd": "/path/to/project",
            "messages": [
                { "role": "user", "content": "implement the login screen", "timestamp": "2026-06-15T09:00:05Z" },
                { "role": "assistant", "content": "Working on it.", "timestamp": "2026-06-15T09:00:10Z" }
            ]
        }"#;
        let session = parse_session(json, Path::new("session-1.json")).unwrap();
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.project_path, Some("/path/to/project".to_string()));
    }

    #[test]
    fn returns_none_for_unparseable_content() {
        assert!(parse_session("not json", Path::new("x.json")).is_none());
    }

    #[test]
    fn returns_none_when_no_messages() {
        let json = r#"{"id": "s1", "cwd": "/x"}"#;
        assert!(parse_session(json, Path::new("s1.json")).is_none());
    }
}
