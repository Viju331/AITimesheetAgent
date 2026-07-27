use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    Jira,
    AzureDevOps,
    Microsoft,
    Slack,
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Provider::Jira => "jira",
            Provider::AzureDevOps => "azure_devops",
            Provider::Microsoft => "microsoft",
            Provider::Slack => "slack",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Provider::Jira => "Jira",
            Provider::AzureDevOps => "Azure DevOps",
            Provider::Microsoft => "Microsoft 365",
            Provider::Slack => "Slack",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationAccount {
    pub id: String,
    pub provider: String,
    pub display_name: String,
    pub account_id: Option<String>,
    pub base_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>,
    pub scopes: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub last_synced_at: Option<String>,
    pub created_at: String,
}

impl IntegrationAccount {
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }

    pub fn is_token_expired(&self) -> bool {
        self.expires_at.as_deref().map(|exp| {
            chrono::DateTime::parse_from_rfc3339(exp)
                .map(|dt| dt.with_timezone(&chrono::Utc) < chrono::Utc::now())
                .unwrap_or(false)
        }).unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub id: String,
    pub account_id: String,
    pub provider: String,
    pub provider_item_id: String,
    pub title: String,
    pub item_type: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub project_key: Option<String>,
    pub project_name: Option<String>,
    pub assigned_to: Option<String>,
    pub due_date: Option<String>,
    pub url: Option<String>,
    pub synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub account_id: String,
    pub provider_event_id: String,
    pub title: String,
    pub event_type: String,
    pub start_time: String,
    pub end_time: String,
    pub event_date: String,
    pub duration_minutes: i64,
    pub attendees: Vec<String>,
    pub is_organizer: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationSignal {
    pub id: String,
    pub account_id: String,
    pub source: String,
    pub signal_type: String,
    pub content: String,
    pub ticket_reference: Option<String>,
    pub signal_date: String,
    pub confidence: f64,
}

/// Daily work context profile assembled from all integration sources (P6-018).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkContextProfile {
    pub date: String,
    pub active_work_items: Vec<WorkItem>,
    pub calendar_events: Vec<CalendarEvent>,
    pub communication_signals: Vec<CommunicationSignal>,
    pub connected_providers: Vec<String>,
    pub total_meeting_minutes: i64,
}

/// Result of a sync operation (P6-003 account management).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub account_id: String,
    pub provider: String,
    pub items_synced: i64,
    pub error: Option<String>,
}

/// Intermediate model used during Microsoft Device Code OAuth flow (P6-002).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftDeviceCodeInfo {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: i64,
    pub interval: i64,
    pub message: String,
}
