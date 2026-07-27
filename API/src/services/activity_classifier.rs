/// Classify a piece of extracted activity text into a high-level category (P3-014).
pub fn classify(text: &str) -> String {
    let lower = text.to_lowercase();

    const RULES: &[(&[&str], &str)] = &[
        (&["fix", "bug", "issue", "crash", "broken", "regression"], "Bug Fix"),
        (&["unit test", "write test", "test coverage", "spec for", "add test"], "Unit Test"),
        (&["refactor", "clean up", "cleanup", "restructure", "reorganize", "simplify"], "Refactor"),
        (&["document", "readme", "docs", "comment the", "write documentation"], "Documentation"),
        (&["research", "evaluate", "compare", "explore options", "look into", "spike"], "Research"),
        (&["meeting", "standup", "stand-up", "sync with", "call with"], "Meeting"),
        (&["investigate", "debug", "diagnose", "root cause", "why is", "why does"], "Investigation"),
    ];

    for (keywords, category) in RULES {
        if keywords.iter().any(|k| lower.contains(k)) {
            return category.to_string();
        }
    }

    "Feature".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_bug_fix() {
        assert_eq!(classify("fix the login crash"), "Bug Fix");
    }

    #[test]
    fn classifies_unit_test() {
        assert_eq!(classify("add test coverage for the parser"), "Unit Test");
    }

    #[test]
    fn classifies_refactor() {
        assert_eq!(classify("refactor the activity extractor"), "Refactor");
    }

    #[test]
    fn defaults_to_feature() {
        assert_eq!(classify("implement the dropdown filter"), "Feature");
    }
}
