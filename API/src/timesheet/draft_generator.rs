use super::models::{StyleProfile, TaskGroupDraft};
use super::summary_engine;
use super::task_grouper::TaskCluster;

/// Apply the user's style profile to every activity in every task cluster,
/// producing the final, styled task groups ready to render or persist (P5-011).
pub fn generate(clusters: &[TaskCluster], style: &StyleProfile) -> Vec<TaskGroupDraft> {
    clusters
        .iter()
        .map(|cluster| TaskGroupDraft {
            title: cluster.title.clone(),
            task_key: cluster.task_key.clone(),
            ticket_reference: cluster.ticket_reference.clone(),
            project_id: cluster.project_id.clone(),
            bullets: cluster.activities.iter().map(|a| summary_engine::generate_bullet(a, style)).collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timesheet::models::ConsolidatedActivity;

    fn cluster() -> TaskCluster {
        TaskCluster {
            title: "Dropdown".to_string(),
            task_key: "dropdown".to_string(),
            ticket_reference: None,
            project_id: Some("p1".to_string()),
            activities: vec![ConsolidatedActivity {
                title: "Implemented Dropdown".to_string(),
                topic: "dropdown".to_string(),
                descriptions: vec![],
                category: "Feature".to_string(),
                confidence_score: 0.6,
                ticket_reference: None,
                source_activity_ids: vec!["a1".to_string()],
                project_id: Some("p1".to_string()),
            }],
        }
    }

    #[test]
    fn generates_one_bullet_per_activity() {
        let style = StyleProfile::default();
        let drafts = generate(&[cluster()], &style);
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].bullets.len(), 1);
        assert_eq!(drafts[0].title, "Dropdown");
    }
}
