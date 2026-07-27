use crate::services::task_detector::{shares_topic, significant_words};
use super::models::ConsolidatedActivity;

#[derive(Debug, Clone)]
pub struct TaskCluster {
    pub title: String,
    pub task_key: String,
    pub ticket_reference: Option<String>,
    pub project_id: Option<String>,
    pub activities: Vec<ConsolidatedActivity>,
}

/// Group consolidated activities into named tasks by topic overlap (P5-004).
/// Unlike Phase 3's per-session task detector, this clusters across the
/// entire day's consolidated activities (AI + Git combined) and is the unit
/// persisted to the `task_groups` table for stable history/regeneration.
/// Bullet styling happens downstream in the draft generator, after grouping.
pub fn group(activities: &[ConsolidatedActivity]) -> Vec<TaskCluster> {
    let mut clusters: Vec<TaskCluster> = Vec::new();

    for activity in activities {
        let words = significant_words(&activity.topic);
        let existing = clusters.iter_mut().find(|c| shares_topic(&significant_words(&c.task_key), &words));

        match existing {
            Some(cluster) => {
                cluster.activities.push(activity.clone());
                if cluster.ticket_reference.is_none() {
                    cluster.ticket_reference = activity.ticket_reference.clone();
                }
            }
            None => {
                clusters.push(TaskCluster {
                    title: title_case(&activity.topic),
                    task_key: activity.topic.clone(),
                    ticket_reference: activity.ticket_reference.clone(),
                    project_id: activity.project_id.clone(),
                    activities: vec![activity.clone()],
                });
            }
        }
    }

    clusters
}

fn title_case(text: &str) -> String {
    text.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn consolidated(title: &str, topic: &str) -> ConsolidatedActivity {
        ConsolidatedActivity {
            title: title.to_string(),
            topic: topic.to_string(),
            descriptions: vec![],
            category: "Feature".to_string(),
            confidence_score: 0.5,
            ticket_reference: None,
            source_activity_ids: vec!["a1".to_string()],
            project_id: Some("p1".to_string()),
        }
    }

    #[test]
    fn groups_activities_under_shared_task() {
        let activities = vec![
            consolidated("Fixed dropdown bug", "dropdown bug"),
            consolidated("Added loading indicator", "loading indicator"),
            consolidated("Fixed dropdown alignment", "dropdown alignment"),
        ];
        let groups = group(&activities);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn task_title_is_title_cased() {
        let activities = vec![consolidated("Implemented admin screen", "admin screen")];
        let groups = group(&activities);
        assert_eq!(groups[0].title, "Admin Screen");
    }
}
