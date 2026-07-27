use super::activity_classifier;

#[derive(Debug, Clone, PartialEq)]
pub struct ExtractedActivity {
    pub title: String,
    pub description: String,
    /// The object/topic of the activity with the leading verb stripped — used
    /// by the task detector to group related activities together.
    pub topic: String,
    pub category: String,
    pub confidence_score: f64,
    pub timestamp: String,
}

const VERB_PAST: &[(&str, &str)] = &[
    ("implement", "Implemented"),
    ("fix", "Fixed"),
    ("add", "Added"),
    ("remove", "Removed"),
    ("delete", "Deleted"),
    ("update", "Updated"),
    ("refactor", "Refactored"),
    ("test", "Tested"),
    ("review", "Reviewed"),
    ("create", "Created"),
    ("build", "Built"),
    ("design", "Designed"),
    ("investigate", "Investigated"),
    ("debug", "Debugged"),
    ("optimize", "Optimized"),
    ("write", "Wrote"),
    ("document", "Documented"),
    ("setup", "Set Up"),
    ("configure", "Configured"),
];

/// Extract a meaningful work activity from one piece of user-prompt text (P3-012).
/// Returns `None` for blank input. Detected verbs raise confidence; otherwise
/// the first sentence is used verbatim at lower confidence.
pub fn extract_from_text(text: &str, timestamp: &str) -> Option<ExtractedActivity> {
    let first_line = first_sentence(text);
    if first_line.is_empty() {
        return None;
    }

    let lower = first_line.to_lowercase();
    let mut matched: Option<(&str, &str, usize)> = None;

    for (verb, past) in VERB_PAST {
        if let Some(pos) = find_word(&lower, verb) {
            if matched.map(|(_, _, p)| pos < p).unwrap_or(true) {
                matched = Some((verb, past, pos));
            }
        }
    }

    let (title, topic, confidence) = match matched {
        Some((verb, past, pos)) => {
            let after = &first_line[pos + verb.len()..];
            let object = after.trim_start_matches(|c: char| !c.is_alphanumeric()).trim();
            if object.is_empty() {
                (past.to_string(), past.to_lowercase(), 0.6)
            } else {
                let title = format!("{} {}", past, title_case(object));
                (truncate(&title, 90), object.to_lowercase(), 0.75)
            }
        }
        None => {
            let title = capitalize_first(&first_line);
            (truncate(&title, 90), first_line.to_lowercase(), 0.4)
        }
    };

    Some(ExtractedActivity {
        title,
        description: first_line.clone(),
        topic,
        category: activity_classifier::classify(&first_line),
        confidence_score: confidence,
        timestamp: timestamp.to_string(),
    })
}

fn find_word(haystack: &str, word: &str) -> Option<usize> {
    let bytes = haystack.as_bytes();
    let wlen = word.len();
    let mut start = 0;
    while start < haystack.len() {
        let Some(idx) = haystack[start..].find(word) else { break };
        let abs = start + idx;
        let before_ok = abs == 0 || !bytes[abs - 1].is_ascii_alphanumeric();
        let after_idx = abs + wlen;
        let after_ok = after_idx >= bytes.len() || !bytes[after_idx].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return Some(abs);
        }
        start = abs + wlen;
    }
    None
}

fn first_sentence(text: &str) -> String {
    let line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    if let Some(idx) = line.find(['.', '?', '!']) {
        if idx > 10 {
            return line[..idx].trim().to_string();
        }
    }
    line.to_string()
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn title_case(s: &str) -> String {
    s.split_whitespace().map(capitalize_first).collect::<Vec<_>>().join(" ")
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() > max_chars {
        let truncated: String = s.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_verb_object_title() {
        let activity = extract_from_text("implement dropdown", "2026-06-15T09:00:00Z").unwrap();
        assert_eq!(activity.title, "Implemented Dropdown");
        assert_eq!(activity.topic, "dropdown");
        assert_eq!(activity.confidence_score, 0.75);
    }

    #[test]
    fn falls_back_when_no_known_verb() {
        let activity = extract_from_text("the dropdown looks weird today", "2026-06-15T09:00:00Z").unwrap();
        assert_eq!(activity.title, "The dropdown looks weird today");
        assert_eq!(activity.confidence_score, 0.4);
    }

    #[test]
    fn returns_none_for_blank_text() {
        assert!(extract_from_text("   ", "2026-06-15T09:00:00Z").is_none());
    }

    #[test]
    fn classifies_bug_fix_via_category() {
        let activity = extract_from_text("fix the login crash", "2026-06-15T09:00:00Z").unwrap();
        assert_eq!(activity.category, "Bug Fix");
    }

    #[test]
    fn uses_first_sentence_only() {
        let activity = extract_from_text(
            "implement the login screen. Also check the password reset flow while you're at it.",
            "2026-06-15T09:00:00Z",
        )
        .unwrap();
        assert_eq!(activity.title, "Implemented The Login Screen");
    }
}
