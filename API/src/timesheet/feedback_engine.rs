use super::models::StyleProfile;

#[derive(Debug, Clone, PartialEq)]
pub struct VerbChange {
    pub original_verb: Option<String>,
    pub edited_verb: Option<String>,
}

/// Compare a generated bullet against the user's edited version and detect
/// whether they swapped the opening verb (e.g. "Implemented" → "Developed")
/// (P5-017). Only the first-word comparison is needed — that's the verb
/// position emitted by [`super::summary_engine`].
pub fn detect_verb_change(original: &str, edited: &str) -> VerbChange {
    VerbChange { original_verb: first_word(original), edited_verb: first_word(edited) }
}

/// Fold a detected verb substitution into the style profile so future
/// generations use the user's preferred wording automatically.
pub fn apply_feedback(profile: &mut StyleProfile, change: &VerbChange) {
    if let (Some(original), Some(edited)) = (&change.original_verb, &change.edited_verb) {
        if original != edited {
            profile.learned_verb_overrides.insert(original.clone(), edited.clone());
        }
    }
}

fn first_word(text: &str) -> Option<String> {
    text.split_whitespace()
        .next()
        .map(|w| w.trim_end_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_verb_substitution() {
        let change = detect_verb_change("Implemented dropdown", "Developed dropdown");
        assert_eq!(change.original_verb, Some("Implemented".to_string()));
        assert_eq!(change.edited_verb, Some("Developed".to_string()));
    }

    #[test]
    fn no_change_when_verb_is_identical() {
        let change = detect_verb_change("Implemented dropdown", "Implemented dropdown filter");
        assert_eq!(change.original_verb, change.edited_verb);
    }

    #[test]
    fn applying_feedback_updates_profile_overrides() {
        let mut profile = StyleProfile::default();
        let change = VerbChange { original_verb: Some("Implemented".to_string()), edited_verb: Some("Developed".to_string()) };
        apply_feedback(&mut profile, &change);
        assert_eq!(profile.learned_verb_overrides.get("Implemented"), Some(&"Developed".to_string()));
    }

    #[test]
    fn identical_verbs_do_not_create_override() {
        let mut profile = StyleProfile::default();
        let change = VerbChange { original_verb: Some("Fixed".to_string()), edited_verb: Some("Fixed".to_string()) };
        apply_feedback(&mut profile, &change);
        assert!(profile.learned_verb_overrides.is_empty());
    }
}
