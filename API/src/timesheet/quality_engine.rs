use super::models::{ConsolidatedActivity, QualityScore};

/// Score a generated draft along four axes (P5-016):
/// - `completeness`: share of today's raw activities that cleared the
///   confidence bar and made it into the draft.
/// - `confidence`: average confidence across included activities.
/// - `duplication`: how much redundant noise consolidation removed from the
///   raw signal (closer to 1.0 = little redundancy existed; cleaner source data).
/// - `coverage`: share of selected projects with at least one represented activity.
pub fn score(
    raw_activity_count: usize,
    consolidated: &[ConsolidatedActivity],
    selected_project_ids: &[String],
) -> QualityScore {
    let included = consolidated.len();

    let completeness = if raw_activity_count == 0 { 0.0 } else { (included as f64 / raw_activity_count as f64).min(1.0) };

    let confidence = if consolidated.is_empty() {
        0.0
    } else {
        consolidated.iter().map(|a| a.confidence_score).sum::<f64>() / consolidated.len() as f64
    };

    let duplication = if raw_activity_count == 0 { 0.0 } else { (included as f64 / raw_activity_count as f64).min(1.0) };

    let coverage = if selected_project_ids.is_empty() {
        0.0
    } else {
        let covered = selected_project_ids
            .iter()
            .filter(|pid| consolidated.iter().any(|a| a.project_id.as_deref() == Some(pid.as_str())))
            .count();
        covered as f64 / selected_project_ids.len() as f64
    };

    let overall = 0.3 * completeness + 0.3 * confidence + 0.2 * duplication + 0.2 * coverage;

    QualityScore { completeness, confidence, duplication, coverage, overall }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn activity(project_id: &str, confidence: f64) -> ConsolidatedActivity {
        ConsolidatedActivity {
            title: "Implemented X".to_string(),
            topic: "x".to_string(),
            descriptions: vec![],
            category: "Feature".to_string(),
            confidence_score: confidence,
            ticket_reference: None,
            source_activity_ids: vec!["a1".to_string()],
            project_id: Some(project_id.to_string()),
        }
    }

    #[test]
    fn full_coverage_and_confidence_yields_high_overall() {
        let consolidated = vec![activity("p1", 1.0), activity("p2", 1.0)];
        let result = score(2, &consolidated, &["p1".to_string(), "p2".to_string()]);
        assert_eq!(result.completeness, 1.0);
        assert_eq!(result.confidence, 1.0);
        assert_eq!(result.coverage, 1.0);
        assert!((result.overall - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn partial_coverage_reduces_overall_score() {
        let consolidated = vec![activity("p1", 0.5)];
        let result = score(1, &consolidated, &["p1".to_string(), "p2".to_string()]);
        assert_eq!(result.coverage, 0.5);
        assert!(result.overall < 1.0);
    }

    #[test]
    fn handles_zero_activities_without_panicking() {
        let result = score(0, &[], &[]);
        assert_eq!(result.overall, 0.0);
    }
}
