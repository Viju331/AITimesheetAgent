use crate::integrations::models::WorkItem;
use crate::timesheet::ticket_extractor;

/// A confirmed link between an activity and a work item.
#[derive(Debug, Clone)]
pub struct ActivityCorrelation {
    pub activity_id: String,
    pub work_item_id: String,
    pub provider_item_id: String,
    pub match_reason: MatchReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchReason {
    /// Activity text directly mentions the ticket ID (e.g. "PROJ-123").
    TicketReference,
    /// Activity title words overlap significantly with the work item title.
    KeywordOverlap,
}

/// Correlate a list of activities (id + text) against known work items.
/// Returns all (activity_id, work_item_id) pairs that can be linked (P6-017).
pub fn correlate(
    activities: &[(String, String)],
    work_items: &[WorkItem],
) -> Vec<ActivityCorrelation> {
    let mut correlations = Vec::new();

    for (activity_id, activity_text) in activities {
        let refs = ticket_extractor::extract_all(activity_text);

        // First pass: exact ticket-reference match.
        for work_item in work_items {
            if refs.iter().any(|r| references_item(r, &work_item.provider_item_id)) {
                correlations.push(ActivityCorrelation {
                    activity_id: activity_id.clone(),
                    work_item_id: work_item.id.clone(),
                    provider_item_id: work_item.provider_item_id.clone(),
                    match_reason: MatchReason::TicketReference,
                });
            }
        }

        // Second pass: keyword overlap (only if no ticket ref was found for this activity).
        let already_linked = correlations.iter().any(|c| c.activity_id == *activity_id);
        if !already_linked {
            let activity_words = significant_words(activity_text);
            if !activity_words.is_empty() {
                for work_item in work_items {
                    let item_words = significant_words(&work_item.title);
                    let overlap = word_overlap_count(&activity_words, &item_words);
                    let min_len = activity_words.len().min(item_words.len());
                    if overlap >= 1 && min_len > 0 && overlap * 2 >= min_len {
                        correlations.push(ActivityCorrelation {
                            activity_id: activity_id.clone(),
                            work_item_id: work_item.id.clone(),
                            provider_item_id: work_item.provider_item_id.clone(),
                            match_reason: MatchReason::KeywordOverlap,
                        });
                    }
                }
            }
        }
    }

    correlations
}

/// Check if a ticket reference string matches a provider item ID.
/// Handles case-insensitive Jira refs and ADO numeric IDs like "AB#123" vs "123".
fn references_item(ticket_ref: &str, provider_item_id: &str) -> bool {
    if ticket_ref.eq_ignore_ascii_case(provider_item_id) {
        return true;
    }
    // ADO: "AB#456" or "#456" → compare numeric part only.
    let ref_num = ticket_ref.trim_start_matches("AB").trim_start_matches('#');
    let item_num = provider_item_id.trim_start_matches("AB").trim_start_matches('#');
    if !ref_num.is_empty() && ref_num == item_num {
        return true;
    }
    false
}

const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "with", "by", "is", "was", "are", "be", "been", "has", "have",
    "had", "do", "does", "did", "will", "would", "could", "should", "may",
    "might", "must", "can", "it", "its", "this", "that", "these", "those",
    "from", "into", "up", "as", "not", "no", "so", "if", "then",
];

fn significant_words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphabetic())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() > 2 && !STOPWORDS.contains(&w.as_str()))
        .collect()
}

fn word_overlap_count(a: &[String], b: &[String]) -> usize {
    a.iter().filter(|w| b.contains(w)).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wi(id: &str, provider_item_id: &str, title: &str) -> WorkItem {
        WorkItem {
            id: id.to_string(),
            account_id: "acc1".to_string(),
            provider: "jira".to_string(),
            provider_item_id: provider_item_id.to_string(),
            title: title.to_string(),
            item_type: None,
            status: None,
            priority: None,
            project_key: None,
            project_name: None,
            assigned_to: None,
            due_date: None,
            url: None,
            synced_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn matches_jira_ticket_reference() {
        let activities = vec![("a1".to_string(), "Fixed login bug PROJ-123".to_string())];
        let items = vec![wi("w1", "PROJ-123", "Login page broken")];
        let correlations = correlate(&activities, &items);
        assert_eq!(correlations.len(), 1);
        assert_eq!(correlations[0].match_reason, MatchReason::TicketReference);
    }

    #[test]
    fn matches_ado_hash_reference() {
        let activities = vec![("a1".to_string(), "Worked on AB#456 deployment".to_string())];
        let items = vec![wi("w1", "456", "Deploy pipeline")];
        let correlations = correlate(&activities, &items);
        assert_eq!(correlations.len(), 1);
        assert_eq!(correlations[0].match_reason, MatchReason::TicketReference);
    }

    #[test]
    fn falls_back_to_keyword_overlap() {
        let activities = vec![("a1".to_string(), "Updated authentication login flow".to_string())];
        let items = vec![wi("w1", "PROJ-999", "Authentication flow redesign")];
        let correlations = correlate(&activities, &items);
        assert_eq!(correlations.len(), 1);
        assert_eq!(correlations[0].match_reason, MatchReason::KeywordOverlap);
    }

    #[test]
    fn no_match_for_unrelated() {
        let activities = vec![("a1".to_string(), "Ran database migration".to_string())];
        let items = vec![wi("w1", "PROJ-001", "Redesign navigation menu")];
        let correlations = correlate(&activities, &items);
        assert!(correlations.is_empty());
    }
}
