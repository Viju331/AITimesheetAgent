use crate::git::models::{CommitInfo, FileCategory, GitChangeType};

const BUGFIX_PREFIXES: &[&str] = &["fix", "bug", "hotfix", "resolve", "patch"];
const FEATURE_PREFIXES: &[&str] = &["feat", "add", "implement", "create", "new"];
const REFACTOR_PREFIXES: &[&str] = &["refactor", "rename", "move", "restructure", "cleanup"];
const TEST_PREFIXES: &[&str] = &["test", "spec", "coverage"];
const DOCS_PREFIXES: &[&str] = &["docs", "doc", "readme", "changelog"];
const CHORE_PREFIXES: &[&str] = &["chore", "build", "ci", "deps", "bump"];

/// Classify a commit into a change type using the rules from
/// `docs/research/git-activity-model.md`: commit-message prefixes are
/// checked first (in priority order), then file-pattern signals as a
/// fallback, defaulting to `Feature` (P4-010/P4-011/P4-012/P4-013).
pub fn classify(commit: &CommitInfo) -> GitChangeType {
    let subject = strip_conventional_prefix(&commit.subject).to_lowercase();

    if starts_with_any(&subject, BUGFIX_PREFIXES) {
        return GitChangeType::BugFix;
    }
    if starts_with_any(&subject, FEATURE_PREFIXES) {
        return GitChangeType::Feature;
    }
    if starts_with_any(&subject, REFACTOR_PREFIXES) {
        return GitChangeType::Refactor;
    }
    if starts_with_any(&subject, TEST_PREFIXES) {
        return GitChangeType::Test;
    }
    if starts_with_any(&subject, DOCS_PREFIXES) {
        return GitChangeType::Docs;
    }
    if starts_with_any(&subject, CHORE_PREFIXES) {
        return GitChangeType::Chore;
    }

    // Fall back to file-pattern signals when the message gives no hint.
    if !commit.files.is_empty() && commit.files.iter().all(|f| f.category == FileCategory::Tests) {
        return GitChangeType::Test;
    }
    if !commit.files.is_empty() && commit.files.iter().all(|f| f.category == FileCategory::Documentation) {
        return GitChangeType::Docs;
    }

    GitChangeType::Feature
}

/// True when the change type was determined from the commit message itself
/// rather than falling back to file-pattern heuristics — used by the
/// confidence engine (P4-016 / research doc contribution table).
pub fn was_classified_from_message(commit: &CommitInfo) -> bool {
    let subject = strip_conventional_prefix(&commit.subject).to_lowercase();
    [BUGFIX_PREFIXES, FEATURE_PREFIXES, REFACTOR_PREFIXES, TEST_PREFIXES, DOCS_PREFIXES, CHORE_PREFIXES]
        .iter()
        .any(|prefixes| starts_with_any(&subject, prefixes))
}

/// Strip a conventional-commit style prefix (`feat:`, `fix(scope):`, etc.)
/// so title generation doesn't double up on the verb.
pub fn strip_conventional_prefix(subject: &str) -> &str {
    match subject.split_once(':') {
        Some((prefix, rest)) => {
            let prefix_word = prefix.split('(').next().unwrap_or(prefix).trim().to_lowercase();
            let all_prefixes = [
                BUGFIX_PREFIXES, FEATURE_PREFIXES, REFACTOR_PREFIXES, TEST_PREFIXES, DOCS_PREFIXES, CHORE_PREFIXES,
            ];
            if all_prefixes.iter().flat_map(|p| p.iter()).any(|p| *p == prefix_word) {
                rest.trim()
            } else {
                subject
            }
        }
        None => subject,
    }
}

/// Build a human-readable activity title from a classified commit, per the
/// title-template table in `docs/research/git-activity-model.md`.
pub fn generate_title(change_type: GitChangeType, subject: &str) -> String {
    let clean = strip_conventional_prefix(subject);
    let clean = capitalize_first(clean);
    match change_type {
        GitChangeType::Feature => format!("Implemented {clean}"),
        GitChangeType::BugFix => format!("Fixed {clean}"),
        GitChangeType::Refactor => format!("Refactored {clean}"),
        GitChangeType::Test => format!("Added tests for {clean}"),
        GitChangeType::Docs => format!("Updated documentation: {clean}"),
        GitChangeType::Chore => clean,
    }
}

fn starts_with_any(text: &str, prefixes: &[&str]) -> bool {
    prefixes.iter().any(|p| text.starts_with(p))
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::models::{FileChangeInfo, FileStatus};

    fn commit(subject: &str) -> CommitInfo {
        CommitInfo {
            hash: "abc123".to_string(),
            author_name: "Test".to_string(),
            author_email: "test@example.com".to_string(),
            commit_date: "2026-06-15T09:00:00Z".to_string(),
            subject: subject.to_string(),
            files: vec![],
        }
    }

    #[test]
    fn classifies_bugfix_by_prefix() {
        assert_eq!(classify(&commit("fix township dropdown")), GitChangeType::BugFix);
    }

    #[test]
    fn classifies_feature_by_prefix() {
        assert_eq!(classify(&commit("add land neighborhood screen")), GitChangeType::Feature);
    }

    #[test]
    fn classifies_refactor_by_prefix() {
        assert_eq!(classify(&commit("refactor activity extractor")), GitChangeType::Refactor);
    }

    #[test]
    fn classifies_test_by_file_pattern_when_message_is_ambiguous() {
        let mut c = commit("township dropdown");
        c.files = vec![FileChangeInfo {
            path: "src/app/township.component.spec.ts".to_string(),
            status: FileStatus::Added,
            category: FileCategory::Tests,
            insertions: 20,
            deletions: 0,
        }];
        assert_eq!(classify(&c), GitChangeType::Test);
    }

    #[test]
    fn defaults_to_feature() {
        assert_eq!(classify(&commit("township dropdown")), GitChangeType::Feature);
    }

    #[test]
    fn strips_conventional_prefix() {
        assert_eq!(strip_conventional_prefix("feat: add login screen"), "add login screen");
        assert_eq!(strip_conventional_prefix("fix(auth): token refresh"), "token refresh");
    }

    #[test]
    fn generates_title_per_template() {
        assert_eq!(generate_title(GitChangeType::BugFix, "fix: township dropdown"), "Fixed Township dropdown");
        assert_eq!(generate_title(GitChangeType::Feature, "add login screen"), "Implemented Add login screen");
    }

    #[test]
    fn detects_message_based_classification() {
        assert!(was_classified_from_message(&commit("fix township dropdown")));
        assert!(!was_classified_from_message(&commit("township dropdown")));
    }
}
