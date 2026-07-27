use crate::git::models::CommitInfo;
use crate::services::git_classifier;

const TICKET_PATTERN_PREFIXES: &[&str] = &["jira-", "ado-"];

/// Score a git-sourced activity that has no corroborating AI session, using
/// the additive contribution table from `docs/research/git-activity-model.md`.
/// Capped below 1.0 — full confidence is reserved for activities corroborated
/// by an AI session (see [`correlated_confidence`]).
pub fn git_only_confidence(commit: &CommitInfo, branch: Option<&str>) -> f64 {
    let mut score = 0.3_f64;

    if commit.subject.trim().len() > 10 {
        score += 0.15;
    }
    if references_ticket(&commit.subject) {
        score += 0.20;
    }
    if git_classifier::was_classified_from_message(commit) {
        score += 0.10;
    }
    if let Some(branch) = branch {
        if !matches!(branch, "main" | "master" | "develop" | "dev") {
            score += 0.10;
        }
    }

    score.min(0.85)
}

fn references_ticket(subject: &str) -> bool {
    let lower = subject.to_lowercase();
    if lower.contains('#') && lower.chars().any(|c| c.is_ascii_digit()) {
        if let Some(idx) = lower.find('#') {
            if lower[idx + 1..].chars().next().is_some_and(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    TICKET_PATTERN_PREFIXES.iter().any(|p| lower.contains(p))
}

/// Confidence ladder for activities correlated across evidence sources
/// (P4-016): AI alone is the weakest signal; AI corroborated by uncommitted
/// git changes is strong; AI corroborated by an actual commit is full confidence.
pub fn correlated_confidence(has_ai: bool, has_git_working_tree: bool, has_git_commit: bool) -> f64 {
    match (has_ai, has_git_commit, has_git_working_tree) {
        (true, true, _) => 1.0,
        (true, false, true) => 0.9,
        (true, false, false) => 0.4,
        (false, _, _) => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::models::CommitInfo;

    fn commit(subject: &str) -> CommitInfo {
        CommitInfo {
            hash: "abc".to_string(),
            author_name: "Test".to_string(),
            author_email: "t@e.com".to_string(),
            commit_date: "2026-06-15T09:00:00Z".to_string(),
            subject: subject.to_string(),
            files: vec![],
        }
    }

    #[test]
    fn ai_only_is_forty_percent() {
        assert_eq!(correlated_confidence(true, false, false), 0.4);
    }

    #[test]
    fn ai_plus_uncommitted_git_is_ninety_percent() {
        assert_eq!(correlated_confidence(true, true, false), 0.9);
    }

    #[test]
    fn ai_plus_commit_is_full_confidence() {
        assert_eq!(correlated_confidence(true, true, true), 1.0);
        assert_eq!(correlated_confidence(true, false, true), 1.0);
    }

    #[test]
    fn git_only_baseline_with_short_unclassifiable_message() {
        let score = git_only_confidence(&commit("wip"), Some("main"));
        assert_eq!(score, 0.3);
    }

    #[test]
    fn git_only_with_ticket_and_message_classification_and_feature_branch() {
        let score = git_only_confidence(&commit("fix township dropdown JIRA-123"), Some("bugfix/dropdown"));
        // base 0.3 + long message 0.15 + ticket 0.20 + message-classified 0.10 + feature branch 0.10 = 0.85
        assert!((score - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn detects_hash_style_ticket_reference() {
        assert!(references_ticket("fix login bug #123"));
        assert!(!references_ticket("fix login bug"));
    }
}
