use base64::Engine as _;
use serde_json::Value;
use crate::errors::{AppError, AppResult};
use super::super::models::WorkItem;

pub struct JiraClient {
    pub base_url: String,
    auth_header: String,
}

impl JiraClient {
    /// Create a Jira client using email + API token (Basic auth) (P6-004).
    pub fn new(base_url: &str, email: &str, api_token: &str) -> Self {
        let credentials = format!("{email}:{api_token}");
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        JiraClient {
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_header: format!("Basic {encoded}"),
        }
    }

    /// Fetch the current user's display name and Jira account ID in one call (P6-004).
    pub fn fetch_myself(&self) -> AppResult<(String, String)> {
        let url = format!("{}/rest/api/3/myself", self.base_url);
        let response: Value = ureq::get(&url)
            .set("Authorization", &self.auth_header)
            .set("Accept", "application/json")
            .call()
            .map_err(|e| AppError::AiProvider(format!("Jira connection failed: {e}")))?
            .into_json()
            .map_err(|e| AppError::Parse(format!("Jira user parse error: {e}")))?;

        let display_name = response["displayName"].as_str()
            .or_else(|| response["emailAddress"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::AiProvider("Jira: could not identify current user".into()))?;
        let account_id = response["accountId"].as_str().unwrap_or("").to_string();
        Ok((display_name, account_id))
    }

    /// Verify the credentials work by fetching the current user (P6-004).
    pub fn verify_connection(&self) -> AppResult<String> {
        let (display_name, _) = self.fetch_myself()?;
        Ok(display_name)
    }

    /// Fetch issues assigned to the current user (P6-005).
    pub fn fetch_assigned_issues(&self, account_id: &str) -> AppResult<Vec<WorkItem>> {
        let jql = "assignee = currentUser() AND resolution = Unresolved ORDER BY updated DESC";
        let url = format!(
            "{}/rest/api/3/search?jql={}&fields=summary,status,priority,project,issuetype,duedate,assignee&maxResults=50",
            self.base_url,
            urlencoding_encode(jql)
        );

        let response: Value = ureq::get(&url)
            .set("Authorization", &self.auth_header)
            .set("Accept", "application/json")
            .call()
            .map_err(|e| AppError::AiProvider(format!("Jira issues fetch failed: {e}")))?
            .into_json()
            .map_err(|e| AppError::Parse(format!("Jira issues parse error: {e}")))?;

        let issues = response["issues"].as_array().cloned().unwrap_or_default();
        let now = chrono::Utc::now().to_rfc3339();

        let work_items = issues.into_iter().map(|issue| {
            let key = issue["key"].as_str().unwrap_or_default().to_string();
            let fields = &issue["fields"];
            WorkItem {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                provider: "jira".to_string(),
                provider_item_id: key.clone(),
                title: fields["summary"].as_str().unwrap_or("Untitled").to_string(),
                item_type: fields["issuetype"]["name"].as_str().map(|s| s.to_string()),
                status: fields["status"]["name"].as_str().map(|s| s.to_string()),
                priority: fields["priority"]["name"].as_str().map(|s| s.to_string()),
                project_key: fields["project"]["key"].as_str().map(|s| s.to_string()),
                project_name: fields["project"]["name"].as_str().map(|s| s.to_string()),
                assigned_to: fields["assignee"]["displayName"].as_str().map(|s| s.to_string()),
                due_date: fields["duedate"].as_str().map(|s| s.to_string()),
                url: Some(format!("{}/browse/{}", self.base_url, key)),
                synced_at: now.clone(),
            }
        }).collect();

        Ok(work_items)
    }

    /// Correlate activities with Jira issues by scanning activity text for
    /// issue keys (P6-006).  Matches: commit messages, branch names, AI prompts.
    pub fn correlate_activities(
        activities: &[crate::repositories::activity_repository::Activity],
        work_items: &[WorkItem],
    ) -> Vec<(String, String)> {
        use crate::timesheet::ticket_extractor;
        let mut correlations: Vec<(String, String)> = Vec::new();

        for activity in activities {
            let search_text = format!(
                "{} {}",
                activity.title,
                activity.description.as_deref().unwrap_or("")
            );
            let refs = ticket_extractor::extract_all(&search_text);
            for ticket_ref in refs {
                if let Some(wi) = work_items.iter().find(|w| w.provider_item_id == ticket_ref) {
                    correlations.push((activity.id.clone(), wi.id.clone()));
                }
            }
        }

        correlations
    }
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            '=' => "%3D".to_string(),
            '(' | ')' => format!("%{:02X}", c as u32),
            c if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') => c.to_string(),
            c => format!("%{:02X}", c as u32),
        })
        .collect()
}
