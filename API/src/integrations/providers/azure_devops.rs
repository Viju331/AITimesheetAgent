use base64::Engine as _;
use serde_json::Value;
use crate::errors::{AppError, AppResult};
use super::super::models::WorkItem;

pub struct AzureDevOpsClient {
    pub organization: String,
    auth_header: String,
}

impl AzureDevOpsClient {
    /// Create an ADO client using a Personal Access Token (P6-007).
    pub fn new(organization: &str, pat_token: &str) -> Self {
        let credentials = format!(":{pat_token}");
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        AzureDevOpsClient {
            organization: organization.to_string(),
            auth_header: format!("Basic {encoded}"),
        }
    }

    fn api(&self, path: &str) -> String {
        format!("https://dev.azure.com/{}/{}", self.organization, path)
    }

    /// Verify the PAT works by fetching the current user profile (P6-007).
    pub fn verify_connection(&self) -> AppResult<String> {
        let url = "https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=7.0";
        let response: Value = ureq::get(url)
            .set("Authorization", &self.auth_header)
            .set("Accept", "application/json")
            .call()
            .map_err(|e| AppError::AiProvider(format!("ADO connection failed: {e}")))?
            .into_json()
            .map_err(|e| AppError::Parse(format!("ADO profile parse error: {e}")))?;

        response["displayName"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::AiProvider("ADO: could not identify current user".into()))
    }

    /// Fetch assigned work items across all projects in the organization (P6-008).
    pub fn fetch_assigned_work_items(&self, account_id: &str) -> AppResult<Vec<WorkItem>> {
        let wiql = r#"{
            "query": "SELECT [System.Id],[System.Title],[System.WorkItemType],[System.State],[System.Priority],[System.AreaPath],[System.TeamProject],[Microsoft.VSTS.Common.Priority] FROM WorkItems WHERE [System.AssignedTo] = @Me AND [System.State] <> 'Closed' AND [System.State] <> 'Resolved' ORDER BY [System.ChangedDate] DESC"
        }"#;

        let url = self.api("_apis/wit/wiql?api-version=7.0");
        let wiql_response: Value = ureq::post(&url)
            .set("Authorization", &self.auth_header)
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .send_string(wiql)
            .map_err(|e| AppError::AiProvider(format!("ADO WIQL query failed: {e}")))?
            .into_json()
            .map_err(|e| AppError::Parse(format!("ADO WIQL parse error: {e}")))?;

        let ids: Vec<i64> = wiql_response["workItems"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|wi| wi["id"].as_i64())
            .take(50)
            .collect();

        if ids.is_empty() {
            return Ok(vec![]);
        }

        let ids_str: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let batch_url = self.api(&format!(
            "_apis/wit/workitemsbatch?api-version=7.0"
        ));

        let batch_body = serde_json::json!({
            "ids": ids,
            "fields": ["System.Id","System.Title","System.WorkItemType","System.State","System.Priority","System.TeamProject","System.AreaPath","Microsoft.VSTS.Common.DuedDate","System.AssignedTo"]
        });

        let batch_response: Value = ureq::post(&batch_url)
            .set("Authorization", &self.auth_header)
            .set("Content-Type", "application/json")
            .set("Accept", "application/json")
            .send_json(batch_body)
            .map_err(|e| AppError::AiProvider(format!("ADO batch fetch failed: {e}")))?
            .into_json()
            .map_err(|e| AppError::Parse(format!("ADO batch parse error: {e}")))?;

        let now = chrono::Utc::now().to_rfc3339();
        let work_items = batch_response["value"].as_array().unwrap_or(&vec![]).iter().map(|item| {
            let fields = &item["fields"];
            let id_num = item["id"].as_i64().unwrap_or(0);
            let project = fields["System.TeamProject"].as_str().unwrap_or("").to_string();
            WorkItem {
                id: uuid::Uuid::new_v4().to_string(),
                account_id: account_id.to_string(),
                provider: "azure_devops".to_string(),
                provider_item_id: format!("#{id_num}"),
                title: fields["System.Title"].as_str().unwrap_or("Untitled").to_string(),
                item_type: fields["System.WorkItemType"].as_str().map(|s| s.to_string()),
                status: fields["System.State"].as_str().map(|s| s.to_string()),
                priority: fields["System.Priority"].as_i64().map(|p| p.to_string()),
                project_key: Some(project.clone()),
                project_name: Some(project.clone()),
                assigned_to: fields["System.AssignedTo"]["displayName"].as_str().map(|s| s.to_string()),
                due_date: fields["Microsoft.VSTS.Common.DuedDate"].as_str().map(|s| s.to_string()),
                url: Some(format!("https://dev.azure.com/{}/_workitems/edit/{}", self.organization, id_num)),
                synced_at: now.clone(),
            }
        }).collect();

        Ok(work_items)
    }
}
