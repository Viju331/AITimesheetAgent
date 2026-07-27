use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedMessage {
    pub role: MessageRole,
    pub text: String,
    /// ISO 8601 UTC timestamp, as reported by the source tool.
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedSession {
    pub tool_name: String,
    pub source_session_id: String,
    pub file_path: String,
    pub project_path: Option<String>,
    pub started_at: String,
    pub ended_at: String,
    pub messages: Vec<NormalizedMessage>,
}

impl NormalizedSession {
    pub fn user_messages(&self) -> impl Iterator<Item = &NormalizedMessage> {
        self.messages.iter().filter(|m| m.role == MessageRole::User)
    }
}
