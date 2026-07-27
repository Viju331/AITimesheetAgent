use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulletChar {
    Dash,
    Asterisk,
    Bullet,
    Numbered,
}

impl BulletChar {
    pub fn as_prefix(&self, index: usize) -> String {
        match self {
            BulletChar::Dash => "- ".to_string(),
            BulletChar::Asterisk => "* ".to_string(),
            BulletChar::Bullet => "\u{2022} ".to_string(),
            BulletChar::Numbered => format!("{}. ", index + 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    Task,
    Project,
    Date,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SentenceFormat {
    WithPeriod,
    WithoutPeriod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadingStyle {
    Dash,
    Colon,
    Brackets,
    Bold,
    Plain,
}

/// Matches `docs/research/style-profile-model.md` exactly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleProfile {
    pub preferred_verbs: Vec<String>,
    pub bullet_char: BulletChar,
    pub group_by: GroupBy,
    pub verbosity: Verbosity,
    pub sentence_format: SentenceFormat,
    pub heading_style: HeadingStyle,
    pub examples: Vec<String>,
    pub source_files: Vec<String>,
    /// Verb substitutions learned from user edits (P5-017), checked before
    /// falling back to `preferred_verbs` frequency ranking.
    pub learned_verb_overrides: std::collections::HashMap<String, String>,
}

impl Default for StyleProfile {
    fn default() -> Self {
        StyleProfile {
            preferred_verbs: vec![
                "Implemented".to_string(),
                "Fixed".to_string(),
                "Added".to_string(),
                "Updated".to_string(),
                "Resolved".to_string(),
            ],
            bullet_char: BulletChar::Dash,
            group_by: GroupBy::Task,
            verbosity: Verbosity::Medium,
            sentence_format: SentenceFormat::WithoutPeriod,
            heading_style: HeadingStyle::Dash,
            examples: Vec::new(),
            source_files: Vec::new(),
            learned_verb_overrides: std::collections::HashMap::new(),
        }
    }
}

/// One de-duplicated unit of work after consolidation (P5-002/P5-003).
#[derive(Debug, Clone)]
pub struct ConsolidatedActivity {
    pub title: String,
    pub topic: String,
    pub descriptions: Vec<String>,
    pub category: String,
    pub confidence_score: f64,
    pub ticket_reference: Option<String>,
    pub source_activity_ids: Vec<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BulletPoint {
    pub text: String,
    pub source_activity_ids: Vec<String>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone)]
pub struct TaskGroupDraft {
    pub title: String,
    pub task_key: String,
    pub ticket_reference: Option<String>,
    pub project_id: Option<String>,
    pub bullets: Vec<BulletPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimesheetDraftJson {
    pub task_groups: Vec<TaskGroupJson>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskGroupJson {
    pub title: String,
    pub ticket_reference: Option<String>,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityScore {
    pub completeness: f64,
    pub confidence: f64,
    pub duplication: f64,
    pub coverage: f64,
    pub overall: f64,
}
