use super::models::{BulletPoint, ConsolidatedActivity, SentenceFormat, StyleProfile, Verbosity};

/// Turn one consolidated activity into a readable bullet, applying the
/// user's learned style: verb substitutions confirmed via past edits
/// (highest priority — P5-017), verbosity level, and sentence punctuation
/// (P5-009/P5-010). Never fabricates detail beyond what was actually
/// extracted from AI sessions or git commits.
pub fn generate_bullet(activity: &ConsolidatedActivity, style: &StyleProfile) -> BulletPoint {
    let mut text = apply_verb_override(&activity.title, style);

    match style.verbosity {
        Verbosity::Low => {}
        Verbosity::Medium => {
            if let Some(ticket) = &activity.ticket_reference {
                text = format!("{text} ({ticket})");
            }
        }
        Verbosity::High => {
            if let Some(ticket) = &activity.ticket_reference {
                text = format!("{text} ({ticket})");
            }
            let extra: Vec<&String> = activity.descriptions.iter().filter(|d| *d != &activity.title).take(2).collect();
            if !extra.is_empty() {
                text = format!("{text} — {}", extra.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("; "));
            }
        }
    }

    text = apply_sentence_format(&text, style.sentence_format);

    BulletPoint {
        text,
        source_activity_ids: activity.source_activity_ids.clone(),
        confidence_score: activity.confidence_score,
    }
}

fn apply_verb_override(title: &str, style: &StyleProfile) -> String {
    let Some((first_word, rest)) = title.split_once(' ') else {
        return title.to_string();
    };
    match style.learned_verb_overrides.get(first_word) {
        Some(replacement) => format!("{replacement} {rest}"),
        None => title.to_string(),
    }
}

fn apply_sentence_format(text: &str, format: SentenceFormat) -> String {
    let trimmed = text.trim_end();
    match format {
        SentenceFormat::WithPeriod => {
            if trimmed.ends_with('.') {
                trimmed.to_string()
            } else {
                format!("{trimmed}.")
            }
        }
        SentenceFormat::WithoutPeriod => trimmed.trim_end_matches('.').to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn activity() -> ConsolidatedActivity {
        ConsolidatedActivity {
            title: "Implemented Dropdown".to_string(),
            topic: "dropdown".to_string(),
            descriptions: vec!["Implemented Dropdown".to_string(), "added search filtering".to_string()],
            category: "Feature".to_string(),
            confidence_score: 0.75,
            ticket_reference: Some("JIRA-123".to_string()),
            source_activity_ids: vec!["a1".to_string()],
            project_id: Some("p1".to_string()),
        }
    }

    #[test]
    fn low_verbosity_keeps_bullet_terse() {
        let style = StyleProfile { verbosity: Verbosity::Low, ..StyleProfile::default() };
        let bullet = generate_bullet(&activity(), &style);
        assert_eq!(bullet.text, "Implemented Dropdown");
    }

    #[test]
    fn medium_verbosity_appends_ticket_reference() {
        let style = StyleProfile { verbosity: Verbosity::Medium, ..StyleProfile::default() };
        let bullet = generate_bullet(&activity(), &style);
        assert_eq!(bullet.text, "Implemented Dropdown (JIRA-123)");
    }

    #[test]
    fn high_verbosity_appends_extra_description() {
        let style = StyleProfile { verbosity: Verbosity::High, ..StyleProfile::default() };
        let bullet = generate_bullet(&activity(), &style);
        assert!(bullet.text.contains("added search filtering"));
    }

    #[test]
    fn applies_period_when_style_requires_it() {
        let style = StyleProfile { sentence_format: SentenceFormat::WithPeriod, verbosity: Verbosity::Low, ..StyleProfile::default() };
        let bullet = generate_bullet(&activity(), &style);
        assert!(bullet.text.ends_with('.'));
    }

    #[test]
    fn applies_learned_verb_override() {
        let mut overrides = HashMap::new();
        overrides.insert("Implemented".to_string(), "Developed".to_string());
        let style = StyleProfile { verbosity: Verbosity::Low, learned_verb_overrides: overrides, ..StyleProfile::default() };
        let bullet = generate_bullet(&activity(), &style);
        assert_eq!(bullet.text, "Developed Dropdown");
    }
}
