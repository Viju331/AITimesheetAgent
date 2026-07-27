use std::path::PathBuf;
use super::claude_reader::ClaudeReader;
use super::codex_reader::CodexReader;
use super::cursor_reader::CursorReader;
use super::reader::SessionReader;

/// Resolve the default session-storage folder for a known AI tool name.
/// Returns `None` for unknown tools or when the platform home directory
/// cannot be determined. New tools can be registered here without touching
/// reader or discovery logic elsewhere.
pub fn default_folder_for(tool_name: &str) -> Option<PathBuf> {
    match tool_name {
        "claude_code" => dirs::home_dir().map(|h| h.join(".claude").join("projects")),
        "cursor" => dirs::config_dir().map(|c| c.join("Cursor").join("User").join("workspaceStorage")),
        "codex" => dirs::home_dir().map(|h| h.join(".codex")),
        _ => None,
    }
}

/// Construct the reader implementation registered for a given tool name.
pub fn reader_for(tool_name: &str) -> Option<Box<dyn SessionReader>> {
    match tool_name {
        "claude_code" => Some(Box::new(ClaudeReader)),
        "cursor" => Some(Box::new(CursorReader)),
        "codex" => Some(Box::new(CodexReader)),
        _ => None,
    }
}
