use regex::Regex;
use std::sync::OnceLock;

fn jira_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"\b[A-Z]{2,10}-\d+\b").unwrap())
}

fn ado_hash_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| Regex::new(r"(?:AB)?#\d+\b").unwrap())
}

/// Extract every distinct work-item reference mentioned in `text`: JIRA-style
/// (`PROJ-123`), Azure DevOps `AB#123` / plain `#123` (P5-006). No live API
/// calls — actual Jira/ADO/Planner connectivity is Phase 6 scope.
pub fn extract_all(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();

    for m in jira_pattern().find_iter(text) {
        let value = m.as_str().to_string();
        if !found.contains(&value) {
            found.push(value);
        }
    }
    for m in ado_hash_pattern().find_iter(text) {
        let value = m.as_str().to_string();
        if !found.contains(&value) {
            found.push(value);
        }
    }

    found
}

/// Convenience wrapper returning just the first reference found, if any.
pub fn extract_first(text: &str) -> Option<String> {
    extract_all(text).into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_jira_style_reference() {
        assert_eq!(extract_first("fix JIRA-123 dropdown bug"), Some("JIRA-123".to_string()));
    }

    #[test]
    fn extracts_azure_devops_reference() {
        assert_eq!(extract_first("resolves AB#456 login issue"), Some("AB#456".to_string()));
    }

    #[test]
    fn extracts_plain_hash_reference() {
        assert_eq!(extract_first("fix login bug #789"), Some("#789".to_string()));
    }

    #[test]
    fn returns_none_when_no_reference_present() {
        assert_eq!(extract_first("fix login bug"), None);
    }

    #[test]
    fn extracts_multiple_distinct_references() {
        let refs = extract_all("Relates to JIRA-100 and also #200");
        assert_eq!(refs, vec!["JIRA-100".to_string(), "#200".to_string()]);
    }

    #[test]
    fn does_not_duplicate_repeated_reference() {
        let refs = extract_all("JIRA-1 then again JIRA-1");
        assert_eq!(refs, vec!["JIRA-1".to_string()]);
    }
}
