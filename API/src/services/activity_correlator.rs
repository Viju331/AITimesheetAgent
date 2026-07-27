use crate::repositories::activity_repository::Activity;
use super::task_detector;

/// Find an existing AI-sourced activity whose topic overlaps with a git
/// commit's subject, so the two pieces of evidence can be merged into one
/// unified activity rather than appearing as duplicates (P4-015).
pub fn find_matching_activity<'a>(commit_subject: &str, candidates: &'a [Activity]) -> Option<&'a Activity> {
    let commit_words = task_detector::significant_words(commit_subject);
    candidates.iter().find(|a| {
        let activity_words = task_detector::significant_words(&a.title);
        task_detector::shares_topic(&commit_words, &activity_words)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn activity(title: &str) -> Activity {
        Activity {
            id: "a1".to_string(),
            project_id: Some("p1".to_string()),
            activity_date: "2026-06-15".to_string(),
            activity_type: "Feature".to_string(),
            title: title.to_string(),
            description: None,
            source_session_id: Some("s1".to_string()),
            source_commit_id: None,
            confidence_score: 0.4,
            is_flagged: false,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn matches_overlapping_topic() {
        let candidates = vec![activity("Implemented Township Dropdown")];
        let result = find_matching_activity("fix township dropdown bug", &candidates);
        assert!(result.is_some());
    }

    #[test]
    fn does_not_match_unrelated_topic() {
        let candidates = vec![activity("Implemented Login Screen")];
        let result = find_matching_activity("fix township dropdown bug", &candidates);
        assert!(result.is_none());
    }

    #[test]
    fn returns_none_for_empty_candidates() {
        assert!(find_matching_activity("fix township dropdown", &[]).is_none());
    }
}
