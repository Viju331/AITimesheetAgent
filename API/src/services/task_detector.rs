#[derive(Debug, Clone, PartialEq)]
pub struct TaskGroup {
    pub task_title: String,
    /// Indices back into the caller's activity slice that belong to this task.
    pub activity_indices: Vec<usize>,
}

const STOPWORDS: &[&str] =
    &["the", "a", "an", "to", "for", "of", "in", "on", "and", "with", "that", "this", "is", "are"];

/// Group related activities under a shared task by comparing their topic
/// (the object/noun-phrase left after the verb is stripped by the activity
/// extractor). Two topics belong to the same task when at least half of the
/// smaller topic's significant words also appear in the other (P3-013) —
/// e.g. "dropdown filter" and "dropdown filter bug" group together, while
/// "login screen" stays separate.
pub fn detect_tasks(topics: &[String]) -> Vec<TaskGroup> {
    let word_sets: Vec<Vec<String>> = topics.iter().map(|t| significant_words(t)).collect();
    let mut groups: Vec<(Vec<String>, TaskGroup)> = Vec::new();

    for (idx, words) in word_sets.iter().enumerate() {
        let matched_group = groups.iter().position(|(rep_words, _)| shares_topic(rep_words, words));

        match matched_group {
            Some(gi) => groups[gi].1.activity_indices.push(idx),
            None => groups.push((
                words.clone(),
                TaskGroup { task_title: title_case(&topics[idx]), activity_indices: vec![idx] },
            )),
        }
    }

    groups.into_iter().map(|(_, g)| g).collect()
}

/// Two topics are considered the same subject when at least half of the
/// smaller topic's significant words also appear in the other. Shared with
/// the Phase 4 activity correlator so AI and Git topics are matched consistently.
pub(crate) fn shares_topic(a: &[String], b: &[String]) -> bool {
    if a.is_empty() || b.is_empty() {
        return false;
    }
    let shared = a.iter().filter(|w| b.contains(w)).count();
    let smaller = a.len().min(b.len());
    shared >= 1 && (shared as f64) >= 0.5 * (smaller as f64)
}

pub(crate) fn significant_words(topic: &str) -> Vec<String> {
    let lower = topic.to_lowercase();
    lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && !STOPWORDS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

fn title_case(topic: &str) -> String {
    topic
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_activities_sharing_a_topic() {
        let topics = vec![
            "dropdown filter".to_string(),
            "login screen".to_string(),
            "dropdown filter bug".to_string(),
        ];
        let groups = detect_tasks(&topics);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].activity_indices, vec![0, 2]);
        assert_eq!(groups[1].activity_indices, vec![1]);
    }

    #[test]
    fn empty_input_yields_no_groups() {
        assert!(detect_tasks(&[]).is_empty());
    }

    #[test]
    fn unrelated_topics_stay_separate() {
        let topics = vec!["login screen".to_string(), "payment gateway".to_string()];
        let groups = detect_tasks(&topics);
        assert_eq!(groups.len(), 2);
    }
}
