use std::collections::HashMap;
use super::models::{BulletChar, GroupBy, HeadingStyle, SentenceFormat, StyleProfile, Verbosity};

/// Analyze raw text from uploaded example timesheets (or accepted historical
/// drafts) and derive a [`StyleProfile`] — opening verbs, bullet character,
/// grouping pattern, verbosity, sentence format, and heading style
/// (P5-007/P5-008), per `docs/research/style-profile-model.md`.
pub fn analyze(text: &str, source_files: Vec<String>) -> StyleProfile {
    let lines: Vec<&str> = text.lines().collect();

    let mut verb_counts: HashMap<String, usize> = HashMap::new();
    let mut bullet_char_counts: HashMap<BulletChar, usize> = HashMap::new();
    let mut word_counts: Vec<usize> = Vec::new();
    let mut with_period = 0usize;
    let mut without_period = 0usize;
    let mut headings: Vec<String> = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((bullet_char, content)) = parse_bullet(trimmed) {
            *bullet_char_counts.entry(bullet_char).or_insert(0) += 1;

            let words: Vec<&str> = content.split_whitespace().collect();
            word_counts.push(words.len());

            if let Some(first_word) = words.first() {
                let verb = first_word.trim_end_matches(|c: char| !c.is_alphanumeric());
                if verb.len() > 1 {
                    *verb_counts.entry(capitalize(verb)).or_insert(0) += 1;
                }
            }

            if content.trim_end().ends_with('.') {
                with_period += 1;
            } else {
                without_period += 1;
            }
        } else {
            headings.push(trimmed.to_string());
        }
    }

    let mut preferred_verbs: Vec<(String, usize)> = verb_counts.into_iter().collect();
    preferred_verbs.sort_by(|a, b| b.1.cmp(&a.1));
    let preferred_verbs: Vec<String> = preferred_verbs.into_iter().map(|(v, _)| v).collect();

    let bullet_char = bullet_char_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(c, _)| c)
        .unwrap_or(BulletChar::Dash);

    let avg_words = if word_counts.is_empty() {
        0.0
    } else {
        word_counts.iter().sum::<usize>() as f64 / word_counts.len() as f64
    };
    let verbosity = if avg_words <= 5.0 {
        Verbosity::Low
    } else if avg_words <= 10.0 {
        Verbosity::Medium
    } else {
        Verbosity::High
    };

    let sentence_format = if with_period >= without_period { SentenceFormat::WithPeriod } else { SentenceFormat::WithoutPeriod };

    let group_by = if headings.is_empty() {
        GroupBy::Flat
    } else if headings.iter().any(|h| looks_like_date(h)) {
        GroupBy::Date
    } else {
        GroupBy::Task
    };

    let heading_style = headings.first().map(|h| detect_heading_style(h)).unwrap_or(HeadingStyle::Dash);

    let examples = extract_examples(text);

    let defaults = StyleProfile::default();

    StyleProfile {
        preferred_verbs: if preferred_verbs.is_empty() { defaults.preferred_verbs } else { preferred_verbs },
        bullet_char,
        group_by,
        verbosity,
        sentence_format,
        heading_style,
        examples,
        source_files,
        learned_verb_overrides: HashMap::new(),
    }
}

fn parse_bullet(line: &str) -> Option<(BulletChar, &str)> {
    if let Some(rest) = line.strip_prefix("- ") {
        return Some((BulletChar::Dash, rest));
    }
    if let Some(rest) = line.strip_prefix("* ") {
        return Some((BulletChar::Asterisk, rest));
    }
    if let Some(rest) = line.strip_prefix('\u{2022}') {
        return Some((BulletChar::Bullet, rest.trim_start()));
    }
    let digits: String = line.chars().take_while(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() {
        let remainder = &line[digits.len()..];
        if let Some(rest) = remainder.strip_prefix(". ") {
            return Some((BulletChar::Numbered, rest));
        }
    }
    None
}

fn looks_like_date(text: &str) -> bool {
    const MONTHS: &[&str] = &[
        "january", "february", "march", "april", "may", "june", "july", "august", "september", "october",
        "november", "december", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
    ];
    let lower = text.to_lowercase();
    MONTHS.iter().any(|m| lower.contains(m)) || regex_date_like(&lower)
}

fn regex_date_like(text: &str) -> bool {
    // Cheap ISO-date sniff (YYYY-MM-DD) without pulling in a date parser here.
    let bytes = text.as_bytes();
    if bytes.len() < 10 {
        return false;
    }
    text.split(['-', '/']).count() == 3 && text.chars().filter(|c| c.is_ascii_digit()).count() >= 6
}

fn detect_heading_style(heading: &str) -> HeadingStyle {
    if heading.contains(" - ") {
        HeadingStyle::Dash
    } else if heading.starts_with("**") && heading.ends_with("**") {
        HeadingStyle::Bold
    } else if heading.starts_with('[') && heading.ends_with(']') {
        HeadingStyle::Brackets
    } else if heading.trim_end().ends_with(':') {
        HeadingStyle::Colon
    } else {
        HeadingStyle::Plain
    }
}

fn extract_examples(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|block| block.trim())
        .filter(|block| block.len() > 10)
        .take(5)
        .map(|block| {
            if block.chars().count() > 240 {
                block.chars().take(237).collect::<String>() + "..."
            } else {
                block.to_string()
            }
        })
        .collect()
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Task - Admin Screen\n- Implemented township dropdown with search\n- Fixed CAMA neighborhood validation\n\nTask - API\n- Added user profile endpoint";

    #[test]
    fn detects_preferred_verbs_in_frequency_order() {
        let profile = analyze(SAMPLE, vec![]);
        assert!(profile.preferred_verbs.contains(&"Implemented".to_string()));
        assert!(profile.preferred_verbs.contains(&"Fixed".to_string()));
        assert!(profile.preferred_verbs.contains(&"Added".to_string()));
    }

    #[test]
    fn detects_dash_bullet_char() {
        let profile = analyze(SAMPLE, vec![]);
        assert_eq!(profile.bullet_char, BulletChar::Dash);
    }

    #[test]
    fn detects_task_grouping_and_dash_heading_style() {
        let profile = analyze(SAMPLE, vec![]);
        assert_eq!(profile.group_by, GroupBy::Task);
        assert_eq!(profile.heading_style, HeadingStyle::Dash);
    }

    #[test]
    fn detects_flat_grouping_when_no_headings() {
        let flat = "- Implemented login\n- Fixed bug\n- Added tests";
        let profile = analyze(flat, vec![]);
        assert_eq!(profile.group_by, GroupBy::Flat);
    }

    #[test]
    fn detects_without_period_sentence_format() {
        let profile = analyze(SAMPLE, vec![]);
        assert_eq!(profile.sentence_format, SentenceFormat::WithoutPeriod);
    }

    #[test]
    fn falls_back_to_defaults_when_text_has_no_bullets() {
        let profile = analyze("just some prose with no bullets at all", vec![]);
        assert_eq!(profile.preferred_verbs, StyleProfile::default().preferred_verbs);
    }
}
