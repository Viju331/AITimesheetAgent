use std::fs;
use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::errors::AppResult;
use super::message_parser::{extract_text, is_tool_result_only};
use super::reader::SessionReader;
use super::session_model::{MessageRole, NormalizedMessage, NormalizedSession};

/// Reads Claude Code session transcripts from `~/.claude/projects/<encoded-path>/<uuid>.jsonl`.
pub struct ClaudeReader;

impl SessionReader for ClaudeReader {
    fn tool_name(&self) -> &'static str {
        "claude_code"
    }

    fn discover(&self, tool_folder: &Path) -> AppResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        if !tool_folder.is_dir() {
            return Ok(files);
        }

        for project_entry in fs::read_dir(tool_folder)?.flatten() {
            let project_dir = project_entry.path();
            if !project_dir.is_dir() {
                continue;
            }
            for session_entry in fs::read_dir(&project_dir)?.flatten() {
                let path = session_entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                    files.push(path);
                }
            }
        }
        Ok(files)
    }

    fn read(&self, source: &Path) -> AppResult<Option<NormalizedSession>> {
        let content = fs::read_to_string(source)?;
        Ok(parse_transcript(&content, source))
    }
}

fn parse_transcript(content: &str, source: &Path) -> Option<NormalizedSession> {
    let mut messages: Vec<NormalizedMessage> = Vec::new();
    let mut cwd: Option<String> = None;
    let mut session_id: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let entry: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let entry_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if entry_type != "user" && entry_type != "assistant" {
            continue;
        }

        if cwd.is_none() {
            cwd = entry.get("cwd").and_then(|c| c.as_str()).map(|s| s.to_string());
        }
        if session_id.is_none() {
            session_id = entry.get("sessionId").and_then(|s| s.as_str()).map(|s| s.to_string());
        }

        let Some(timestamp) = entry.get("timestamp").and_then(|t| t.as_str()) else {
            continue;
        };

        let Some(message_content) = entry.get("message").and_then(|m| m.get("content")) else {
            continue;
        };

        if entry_type == "user" && is_tool_result_only(message_content) {
            continue;
        }

        let Some(text) = extract_text(message_content) else {
            continue;
        };

        let role = if entry_type == "user" { MessageRole::User } else { MessageRole::Assistant };
        messages.push(NormalizedMessage { role, text, timestamp: timestamp.to_string() });
    }

    if messages.is_empty() {
        return None;
    }

    messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let started_at = messages.first().unwrap().timestamp.clone();
    let ended_at = messages.last().unwrap().timestamp.clone();

    Some(NormalizedSession {
        tool_name: "claude_code".to_string(),
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
    fn parses_user_and_assistant_entries() {
        let jsonl = r#"
{"type":"queue-operation","operation":"enqueue","timestamp":"2026-06-15T10:55:00.030Z","sessionId":"s1"}
{"parentUuid":null,"type":"user","message":{"role":"user","content":[{"type":"text","text":"implement login screen"}]},"uuid":"u1","timestamp":"2026-06-15T10:55:00.293Z","cwd":"d:\\Project","sessionId":"s1"}
{"parentUuid":"u1","type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Sure, implementing now."}]},"uuid":"a1","timestamp":"2026-06-15T10:55:05.000Z","cwd":"d:\\Project","sessionId":"s1"}
"#;
        let session = parse_transcript(jsonl, Path::new("s1.jsonl")).unwrap();
        assert_eq!(session.messages.len(), 2);
        assert_eq!(session.project_path, Some("d:\\Project".to_string()));
        assert_eq!(session.source_session_id, "s1");
        assert_eq!(session.messages[0].role, MessageRole::User);
        assert_eq!(session.messages[0].text, "implement login screen");
    }

    #[test]
    fn skips_tool_result_entries_disguised_as_user() {
        let jsonl = r#"
{"parentUuid":null,"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t1","content":"file contents here"}]},"uuid":"u1","timestamp":"2026-06-15T10:55:00.293Z","cwd":"d:\\Project","sessionId":"s1"}
{"parentUuid":"u1","type":"user","message":{"role":"user","content":[{"type":"text","text":"real prompt"}]},"uuid":"u2","timestamp":"2026-06-15T10:56:00.293Z","cwd":"d:\\Project","sessionId":"s1"}
"#;
        let session = parse_transcript(jsonl, Path::new("s1.jsonl")).unwrap();
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].text, "real prompt");
    }

    #[test]
    fn returns_none_when_no_usable_entries() {
        let jsonl = r#"{"type":"queue-operation","operation":"enqueue","timestamp":"2026-06-15T10:55:00.030Z","sessionId":"s1"}"#;
        assert!(parse_transcript(jsonl, Path::new("s1.jsonl")).is_none());
    }
}
