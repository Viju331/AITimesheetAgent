use crate::services::task_detector::significant_words;
use super::models::TaskGroupDraft;

#[derive(Debug, Clone)]
pub struct FeatureGroup {
    pub title: String,
    pub task_titles: Vec<String>,
}

/// A coarser, second-tier grouping over task groups for dashboard display
/// (P5-005) — tasks are folded into a feature when they share their leading
/// significant word (e.g. "Admin Screen" and "Admin Validation" → "Admin").
pub fn group(task_groups: &[TaskGroupDraft]) -> Vec<FeatureGroup> {
    let mut features: Vec<FeatureGroup> = Vec::new();

    for task in task_groups {
        let words = significant_words(&task.task_key);
        let Some(lead_word) = words.first() else {
            continue;
        };

        let existing = features.iter_mut().find(|f| {
            significant_words(&f.title.to_lowercase()).first() == Some(lead_word)
        });

        match existing {
            Some(feature) => feature.task_titles.push(task.title.clone()),
            None => {
                features.push(FeatureGroup {
                    title: capitalize(lead_word),
                    task_titles: vec![task.title.clone()],
                });
            }
        }
    }

    features
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

    fn task(title: &str, key: &str) -> TaskGroupDraft {
        TaskGroupDraft {
            title: title.to_string(),
            task_key: key.to_string(),
            ticket_reference: None,
            project_id: None,
            bullets: vec![],
        }
    }

    #[test]
    fn folds_tasks_sharing_lead_word_into_one_feature() {
        let tasks = vec![task("Admin Screen", "admin screen"), task("Admin Validation", "admin validation")];
        let features = group(&tasks);
        assert_eq!(features.len(), 1);
        assert_eq!(features[0].task_titles.len(), 2);
    }

    #[test]
    fn separates_unrelated_tasks() {
        let tasks = vec![task("Admin Screen", "admin screen"), task("Payment Gateway", "payment gateway")];
        let features = group(&tasks);
        assert_eq!(features.len(), 2);
    }
}
