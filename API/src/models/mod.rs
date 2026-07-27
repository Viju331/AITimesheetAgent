use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiToolDto {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub session_folder: Option<String>,
    pub is_enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub tech_stack: Option<String>,
    pub has_git: bool,
    pub git_remote: Option<String>,
    pub is_selected: bool,
    pub created_at: String,
    pub last_scanned: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingStateDto {
    pub has_user: bool,
    pub has_ai_tool: bool,
    pub has_projects: bool,
    pub is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDto {
    pub id: String,
    pub project_id: Option<String>,
    pub tool_id: Option<String>,
    pub session_date: String,
    pub file_path: String,
    pub raw_summary: Option<String>,
    pub message_count: i64,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub parsed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDto {
    pub id: String,
    pub project_id: Option<String>,
    pub activity_date: String,
    pub activity_type: String,
    pub title: String,
    pub description: Option<String>,
    pub source_session_id: Option<String>,
    pub source_commit_id: Option<String>,
    pub confidence_score: f64,
    pub is_flagged: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDto {
    pub id: String,
    pub project_id: String,
    pub root_path: String,
    pub current_branch: Option<String>,
    pub last_checkout_at: Option<String>,
    pub last_scanned_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDto {
    pub id: String,
    pub project_id: String,
    pub commit_hash: String,
    pub branch: Option<String>,
    pub commit_date: String,
    pub commit_message: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub files_changed: i64,
    pub insertions: i64,
    pub deletions: i64,
    pub change_type: Option<String>,
    pub scanned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChangeDto {
    pub id: String,
    pub commit_id: Option<String>,
    pub repository_id: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub extension: Option<String>,
    pub folder: Option<String>,
    pub change_type: String,
    pub category: String,
    pub insertions: i64,
    pub deletions: i64,
    pub is_working_tree: bool,
    pub detected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitDashboardDto {
    pub repository_count: i64,
    pub commit_count: i64,
    pub modified_file_count: i64,
    pub activity_count: i64,
    pub repositories: Vec<RepositoryDto>,
    pub recent_commits: Vec<CommitDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEventDto {
    pub id: String,
    pub project_id: Option<String>,
    pub session_id: Option<String>,
    pub activity_id: Option<String>,
    pub event_date: String,
    pub event_time: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDashboardDto {
    pub session_count: i64,
    pub message_count: i64,
    pub activity_count: i64,
    pub timeline: Vec<TimelineEventDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimesheetDto {
    pub id: String,
    pub user_id: Option<String>,
    pub generated_date: String,
    pub content: String,
    pub raw_draft: Option<String>,
    pub status: String,
    pub activity_ids: Option<String>,
    pub created_at: String,
    pub exported_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleProfileDto {
    pub preferred_verbs: Vec<String>,
    pub bullet_char: String,
    pub group_by: String,
    pub verbosity: String,
    pub sentence_format: String,
    pub heading_style: String,
    pub examples: Vec<String>,
    pub source_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityScoreDto {
    pub completeness: f64,
    pub confidence: f64,
    pub duplication: f64,
    pub coverage: f64,
    pub overall: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTimesheetResponseDto {
    pub timesheet_id: String,
    pub task_group_count: i64,
    pub bullet_count: i64,
    pub quality_score: QualityScoreDto,
    pub plain_text: String,
    pub markdown: String,
    pub html: String,
    pub content_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGroupDto {
    pub id: String,
    pub project_id: Option<String>,
    pub group_date: String,
    pub title: String,
    pub task_key: String,
    pub ticket_reference: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureGroupDto {
    pub title: String,
    pub task_titles: Vec<String>,
}

// ── Integration DTOs (Phase 6) ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationAccountDto {
    pub id: String,
    pub provider: String,
    pub display_name: String,
    pub account_id: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub last_synced_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItemDto {
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
    pub due_date: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEventDto {
    pub id: String,
    pub title: String,
    pub event_type: String,
    pub start_time: String,
    pub end_time: String,
    pub event_date: String,
    pub duration_minutes: i64,
    pub is_organizer: bool,
    pub attendees: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunicationSignalDto {
    pub id: String,
    pub source: String,
    pub signal_type: String,
    pub content: String,
    pub ticket_reference: Option<String>,
    pub signal_date: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkContextProfileDto {
    pub date: String,
    pub active_work_items: Vec<WorkItemDto>,
    pub calendar_events: Vec<CalendarEventDto>,
    pub communication_signals: Vec<CommunicationSignalDto>,
    pub connected_providers: Vec<String>,
    pub total_meeting_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResultDto {
    pub account_id: String,
    pub provider: String,
    pub items_synced: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrosoftDeviceCodeDto {
    pub device_code: String,
    pub user_code: String,
    pub verification_url: String,
    pub expires_in: i64,
    pub interval: i64,
    pub message: String,
}
