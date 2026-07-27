use serde_json::Value;
use crate::errors::{AppError, AppResult};
use super::super::models::{CalendarEvent, CommunicationSignal, WorkItem};

const GRAPH_BASE: &str = "https://graph.microsoft.com/v1.0";

pub struct GraphClient {
    access_token: String,
}

impl GraphClient {
    pub fn new(access_token: &str) -> Self {
        GraphClient { access_token: access_token.to_string() }
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    fn get(&self, path: &str) -> AppResult<Value> {
        let url = format!("{GRAPH_BASE}{path}");
        ureq::get(&url)
            .set("Authorization", &self.auth())
            .set("Accept", "application/json")
            .call()
            .map_err(|e| AppError::AiProvider(format!("Graph API request failed ({path}): {e}")))?
            .into_json()
            .map_err(|e| AppError::Parse(format!("Graph API parse error ({path}): {e}")))
    }

    /// Verify the access token by fetching the current user (P6-009/013/015).
    pub fn verify_connection(&self) -> AppResult<String> {
        let response = self.get("/me")?;
        response["displayName"]
            .as_str()
            .or_else(|| response["mail"].as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::AiProvider("Microsoft Graph: could not identify current user".into()))
    }

    // -----------------------------------------------------------------------
    // Microsoft Planner (P6-009/010)
    // -----------------------------------------------------------------------

    /// Fetch Planner tasks assigned to the current user across all plans (P6-010).
    pub fn fetch_planner_tasks(&self, account_id: &str) -> AppResult<Vec<WorkItem>> {
        let response = self.get("/me/planner/tasks?$top=50&$expand=details,assignedUsers")?;
        let now = chrono::Utc::now().to_rfc3339();

        let tasks = response["value"].as_array().cloned().unwrap_or_default();
        let work_items = tasks.iter().map(|task| {
            let task_id = task["id"].as_str().unwrap_or_default().to_string();
            let plan_id = task["planId"].as_str().unwrap_or_default().to_string();
            WorkItem {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                provider: "planner".to_string(),
                provider_item_id: task_id.clone(),
                title: task["title"].as_str().unwrap_or("Untitled task").to_string(),
                item_type: Some("task".to_string()),
                status: task["percentComplete"].as_i64().map(|p| {
                    if p == 100 { "completed".to_string() } else { "active".to_string() }
                }),
                priority: task["priority"].as_i64().map(|p| p.to_string()),
                project_key: Some(plan_id.clone()),
                project_name: task["bucketId"].as_str().map(|s| s.to_string()),
                assigned_to: None,
                due_date: task["dueDateTime"].as_str().map(|s| s[..10].to_string()),
                url: Some(format!("https://tasks.office.com/tasks/{task_id}")),
                synced_at: now.clone(),
            }
        }).collect();

        Ok(work_items)
    }

    // -----------------------------------------------------------------------
    // Microsoft Teams (P6-013/014)
    // -----------------------------------------------------------------------

    /// Fetch recent Teams messages where the current user was mentioned (P6-014).
    pub fn fetch_teams_signals(&self, account_id: &str, date: &str) -> AppResult<Vec<CommunicationSignal>> {
        let response = match self.get("/me/chats?$expand=lastMessagePreview&$top=20") {
            Ok(r) => r,
            Err(_) => return Ok(vec![]),
        };

        let now = chrono::Utc::now().to_rfc3339();
        let mut signals = Vec::new();

        if let Some(chats) = response["value"].as_array() {
            for chat in chats {
                if let Some(preview) = chat["lastMessagePreview"]["body"]["content"].as_str() {
                    let content = preview.trim().to_string();
                    if content.is_empty() { continue; }

                    let ticket_ref = crate::timesheet::ticket_extractor::extract_first(&content);
                    let confidence = if ticket_ref.is_some() { 0.8 } else { 0.3 };

                    signals.push(CommunicationSignal {
                        id: uuid::Uuid::new_v4().to_string(),
                        account_id: account_id.to_string(),
                        source: "teams".to_string(),
                        signal_type: if ticket_ref.is_some() { "ticket_reference" } else { "chat_context" }.to_string(),
                        content,
                        ticket_reference: ticket_ref,
                        signal_date: date.to_string(),
                        confidence,
                        synced_at: now.clone(),
                    });
                }
            }
        }

        Ok(signals)
    }

    // -----------------------------------------------------------------------
    // Calendar (P6-015/016) — works with Outlook and Microsoft 365 Calendar
    // -----------------------------------------------------------------------

    /// Fetch today's calendar events (P6-016).
    pub fn fetch_calendar_events(&self, account_id: &str, date: &str) -> AppResult<Vec<CalendarEvent>> {
        let start = format!("{date}T00:00:00Z");
        let end = format!("{date}T23:59:59Z");
        let path = format!(
            "/me/calendarView?startDateTime={}&endDateTime={}&$select=id,subject,start,end,isOrganizer,attendees,showAs&$top=50",
            start, end
        );
        let response = self.get(&path)?;

        let now = chrono::Utc::now().to_rfc3339();
        let events = response["value"].as_array().cloned().unwrap_or_default();

        let calendar_events = events.iter().map(|evt| {
            let event_id = evt["id"].as_str().unwrap_or_default().to_string();
            let title = evt["subject"].as_str().unwrap_or("Meeting").to_string();
            let start_time = evt["start"]["dateTime"].as_str().unwrap_or("").to_string();
            let end_time = evt["end"]["dateTime"].as_str().unwrap_or("").to_string();
            let is_organizer = evt["isOrganizer"].as_bool().unwrap_or(false);
            let show_as = evt["showAs"].as_str().unwrap_or("busy");

            let event_type = if show_as == "free" {
                "focus".to_string()
            } else if title.to_lowercase().contains("standup") || title.to_lowercase().contains("sprint") {
                "meeting".to_string()
            } else {
                "meeting".to_string()
            };

            let attendees: Vec<String> = evt["attendees"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|a| a["emailAddress"]["name"].as_str().map(|s| s.to_string()))
                .collect();

            let duration_minutes = calculate_duration_minutes(&start_time, &end_time);

            CalendarEvent {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                provider_event_id: event_id,
                title,
                event_type,
                start_time: start_time.clone(),
                end_time: end_time.clone(),
                event_date: date.to_string(),
                duration_minutes,
                attendees,
                is_organizer,
            }
        }).collect();

        Ok(calendar_events)
    }
}

fn calculate_duration_minutes(start: &str, end: &str) -> i64 {
    let parse = |s: &str| chrono::DateTime::parse_from_rfc3339(s).ok()
        .or_else(|| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
            .ok()
            .map(|dt| dt.and_utc().fixed_offset()));
    match (parse(start), parse(end)) {
        (Some(s), Some(e)) => (e - s).num_minutes().max(0),
        _ => 0,
    }
}
