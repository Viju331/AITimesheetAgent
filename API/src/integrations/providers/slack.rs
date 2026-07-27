use serde_json::Value;
use crate::errors::{AppError, AppResult};
use super::super::models::CommunicationSignal;

pub struct SlackClient {
    token: String,
}

impl SlackClient {
    /// Create a Slack client from a bot or user OAuth token (P6-011).
    pub fn new(token: &str) -> Self {
        SlackClient { token: token.to_string() }
    }

    fn get(&self, path: &str) -> AppResult<Value> {
        let url = format!("https://slack.com/api/{path}");
        ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("Accept", "application/json")
            .call()
            .map_err(|e| AppError::AiProvider(format!("Slack API request failed ({path}): {e}")))?
            .into_json()
            .map_err(|e| AppError::Parse(format!("Slack API parse error ({path}): {e}")))
    }

    /// Verify the token and return the workspace name + user identity (P6-011).
    pub fn verify_connection(&self) -> AppResult<String> {
        let response = self.get("auth.test")?;
        if !response["ok"].as_bool().unwrap_or(false) {
            let error = response["error"].as_str().unwrap_or("unknown error");
            return Err(AppError::AiProvider(format!("Slack auth failed: {error}")));
        }
        let user = response["user"].as_str().unwrap_or("unknown");
        let team = response["team"].as_str().unwrap_or("unknown");
        Ok(format!("{user} @ {team}"))
    }

    /// Fetch DMs and mentions for the current user and extract work context
    /// signals (P6-012). Returns ticket references and work-related messages.
    pub fn fetch_work_signals(&self, account_id: &str, date: &str) -> AppResult<Vec<CommunicationSignal>> {
        let oldest = date_to_epoch(date, 0);
        let latest = date_to_epoch(date, 86400);

        let channels_response = match self.get("conversations.list?limit=10&types=im") {
            Ok(r) => r,
            Err(_) => return Ok(vec![]),
        };

        let now = chrono::Utc::now().to_rfc3339();
        let mut signals: Vec<CommunicationSignal> = Vec::new();

        let channels = channels_response["channels"].as_array().cloned().unwrap_or_default();

        for channel in channels.iter().take(5) {
            let Some(channel_id) = channel["id"].as_str() else { continue };
            let path = format!(
                "conversations.history?channel={}&oldest={}&latest={}&limit=50",
                channel_id, oldest, latest
            );
            let history = match self.get(&path) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let messages = history["messages"].as_array().cloned().unwrap_or_default();
            for msg in &messages {
                let Some(text) = msg["text"].as_str() else { continue };
                let text = text.trim();
                if text.is_empty() { continue; }

                let ticket_ref = crate::timesheet::ticket_extractor::extract_first(text);
                let is_work_related = ticket_ref.is_some()
                    || is_work_keyword(text);

                if !is_work_related { continue; }

                let confidence = if ticket_ref.is_some() { 0.85 } else { 0.4 };
                signals.push(CommunicationSignal {
                    id: uuid::Uuid::new_v4().to_string(),
                    account_id: account_id.to_string(),
                    source: "slack".to_string(),
                    signal_type: if ticket_ref.is_some() {
                        "ticket_reference".to_string()
                    } else {
                        "work_mention".to_string()
                    },
                    content: text.chars().take(500).collect(),
                    ticket_reference: ticket_ref,
                    signal_date: date.to_string(),
                    confidence,
                });
            }
        }

        signals.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        Ok(signals)
    }
}

fn is_work_keyword(text: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "fix", "bug", "implement", "deploy", "review", "pr ", "pull request",
        "sprint", "ticket", "blocker", "build", "feature", "release",
    ];
    let lower = text.to_lowercase();
    KEYWORDS.iter().any(|k| lower.contains(k))
}

fn date_to_epoch(date: &str, offset_secs: i64) -> i64 {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp() + offset_secs)
        .unwrap_or(0)
}
