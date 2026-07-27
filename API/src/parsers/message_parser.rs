use serde_json::Value;

/// Extract human-readable text from a `content` field that may be either a
/// plain string or an array of content blocks (Claude/Codex style). Only
/// `"text"` blocks are considered meaningful — `thinking`, `tool_use`, and
/// `tool_result` blocks are skipped.
pub fn extract_text(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Value::Array(blocks) => {
            let mut parts = Vec::new();
            for block in blocks {
                if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            parts.push(trimmed.to_string());
                        }
                    }
                }
            }
            if parts.is_empty() {
                None
            } else {
                Some(parts.join("\n"))
            }
        }
        _ => None,
    }
}

/// True when a content block array consists entirely of `tool_result` blocks,
/// meaning the entry is a tool response echoed back as a "user" turn rather
/// than genuine developer intent.
pub fn is_tool_result_only(content: &Value) -> bool {
    match content {
        Value::Array(blocks) if !blocks.is_empty() => blocks
            .iter()
            .all(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result")),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_plain_string() {
        assert_eq!(extract_text(&json!("hello")), Some("hello".to_string()));
    }

    #[test]
    fn extracts_text_blocks_only() {
        let content = json!([
            { "type": "thinking", "thinking": "internal" },
            { "type": "text", "text": "implement login" },
        ]);
        assert_eq!(extract_text(&content), Some("implement login".to_string()));
    }

    #[test]
    fn returns_none_for_empty_string() {
        assert_eq!(extract_text(&json!("   ")), None);
    }

    #[test]
    fn detects_tool_result_only() {
        let content = json!([{ "type": "tool_result", "content": "file contents" }]);
        assert!(is_tool_result_only(&content));
    }

    #[test]
    fn mixed_blocks_are_not_tool_result_only() {
        let content = json!([
            { "type": "tool_result", "content": "x" },
            { "type": "text", "text": "y" },
        ]);
        assert!(!is_tool_result_only(&content));
    }
}
