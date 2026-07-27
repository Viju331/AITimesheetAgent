use crate::repositories::activity_repository::Activity;
use crate::services::task_detector::{shares_topic, significant_words};
use super::models::ConsolidatedActivity;
use super::ticket_extractor;

/// Cluster same-topic activities (regardless of source — AI session, git
/// commit, or both) into one consolidated unit so "Implemented dropdown",
/// "Fixed dropdown", and a git commit about the same dropdown don't appear
/// as three separate timesheet lines (P5-002/P5-003).
pub fn consolidate(activities: &[Activity]) -> Vec<ConsolidatedActivity> {
    let mut clusters: Vec<(Vec<String>, Vec<&Activity>)> = Vec::new();

    for activity in activities {
        let words = significant_words(&activity.title);
        let cluster = clusters.iter_mut().find(|(rep_words, _)| shares_topic(rep_words, &words));

        match cluster {
            Some((_, members)) => members.push(activity),
            None => clusters.push((words, vec![activity])),
        }
    }

    clusters.into_iter().map(|(_, members)| merge_cluster(&members)).collect()
}

fn merge_cluster(members: &[&Activity]) -> ConsolidatedActivity {
    let representative = members
        .iter()
        .max_by(|a, b| a.confidence_score.partial_cmp(&b.confidence_score).unwrap())
        .expect("cluster is never empty");

    let mut descriptions: Vec<String> = Vec::new();
    let mut combined_text = String::new();
    for member in members {
        if let Some(desc) = &member.description {
            combined_text.push_str(desc);
            combined_text.push(' ');
            if !descriptions.contains(desc) {
                descriptions.push(desc.clone());
            }
        }
    }
    combined_text.push_str(&representative.title);

    let max_confidence = members.iter().map(|a| a.confidence_score).fold(0.0, f64::max);
    let source_ids: Vec<String> = members.iter().map(|a| a.id.clone()).collect();
    let topic = significant_words(&representative.title).join(" ");

    ConsolidatedActivity {
        title: representative.title.clone(),
        topic,
        descriptions,
        category: representative.activity_type.clone(),
        confidence_score: max_confidence,
        ticket_reference: ticket_extractor::extract_first(&combined_text),
        source_activity_ids: source_ids,
        project_id: representative.project_id.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn activity(id: &str, title: &str, confidence: f64) -> Activity {
        Activity {
            id: id.to_string(),
            project_id: Some("p1".to_string()),
            activity_date: "2026-06-15".to_string(),
            activity_type: "Feature".to_string(),
            title: title.to_string(),
            description: Some(title.to_string()),
            source_session_id: Some("s1".to_string()),
            source_commit_id: None,
            confidence_score: confidence,
            is_flagged: false,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn merges_duplicate_topic_activities() {
        let activities = vec![
            activity("a1", "Implemented dropdown", 0.4),
            activity("a2", "Created dropdown", 0.5),
            activity("a3", "Added dropdown", 0.75),
        ];
        let consolidated = consolidate(&activities);
        assert_eq!(consolidated.len(), 1);
        assert_eq!(consolidated[0].title, "Added dropdown");
        assert_eq!(consolidated[0].confidence_score, 0.75);
        assert_eq!(consolidated[0].source_activity_ids.len(), 3);
    }

    #[test]
    fn keeps_unrelated_activities_separate() {
        let activities = vec![activity("a1", "Implemented login screen", 0.5), activity("a2", "Fixed payment bug", 0.6)];
        let consolidated = consolidate(&activities);
        assert_eq!(consolidated.len(), 2);
    }

    #[test]
    fn extracts_ticket_reference_from_combined_text() {
        let activities = vec![activity("a1", "Fixed JIRA-123 dropdown bug", 0.6)];
        let consolidated = consolidate(&activities);
        assert_eq!(consolidated[0].ticket_reference, Some("JIRA-123".to_string()));
    }
}
