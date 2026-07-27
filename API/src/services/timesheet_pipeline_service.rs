use rusqlite::Connection;
use crate::errors::AppResult;
use crate::parsers::timestamp_engine;
use crate::repositories::{
    activity_repository, project_repository, style_profile_repository, task_group_repository,
    timesheet_feedback_repository, timesheet_repository, user_repository,
};
use crate::timesheet::{
    activity_consolidator, draft_generator, feature_grouper, models::StyleProfile,
    quality_engine, renderer, task_grouper,
};

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
    pub timesheet_id: String,
    pub task_group_count: i64,
    pub bullet_count: i64,
    pub quality_score: crate::timesheet::models::QualityScore,
    pub plain_text: String,
    pub markdown: String,
    pub html: String,
    pub content_json: String,
}

/// Full timesheet generation pipeline (P5-001 flow):
/// activities → consolidate → group → style → draft → quality → store (P5-011).
pub fn generate(conn: &Connection, date: Option<&str>) -> AppResult<GenerateResult> {
    let today = timestamp_engine::today_date_string();
    let date = date.unwrap_or(&today);

    let selected_projects = project_repository::get_selected(conn)?;
    let selected_project_ids: Vec<String> = selected_projects.iter().map(|p| p.id.clone()).collect();

    let all_activities = activity_repository::get_by_date(conn, date)?;
    let raw_count = all_activities.len();

    let style = load_style(conn)?;

    let consolidated = activity_consolidator::consolidate(&all_activities);

    task_group_repository::delete_by_date(conn, date)?;
    let clusters = task_grouper::group(&consolidated);

    let user_id = user_repository::get(conn)?.map(|u| u.id);

    for cluster in &clusters {
        task_group_repository::insert(
            conn,
            cluster.project_id.as_deref(),
            date,
            &cluster.title,
            &cluster.task_key,
            cluster.ticket_reference.as_deref(),
        )?;
    }

    let task_groups = draft_generator::generate(&clusters, &style);
    let quality = quality_engine::score(raw_count, &consolidated, &selected_project_ids);

    let plain_text = renderer::to_plain_text(&task_groups, &style);
    let markdown = renderer::to_markdown(&task_groups, &style);
    let html = renderer::to_html(&task_groups);

    let content_json = build_content_json(&task_groups);

    let bullet_count: i64 = task_groups.iter().map(|g| g.bullets.len() as i64).sum();

    let activity_ids: Vec<&str> = consolidated.iter().flat_map(|c| c.source_activity_ids.iter().map(|id| id.as_str())).collect();
    let activity_ids_json = serde_json::to_string(&activity_ids).unwrap_or_default();

    let stored = timesheet_repository::insert(conn, user_id.as_deref(), date, &content_json, &content_json, &activity_ids_json)?;

    Ok(GenerateResult {
        timesheet_id: stored.id,
        task_group_count: task_groups.len() as i64,
        bullet_count,
        quality_score: quality,
        plain_text,
        markdown,
        html,
        content_json,
    })
}

/// Regenerate the draft for an existing timesheet or date, preserving history
/// by always creating a NEW version (P5-015).
pub fn regenerate(conn: &Connection, date: &str) -> AppResult<GenerateResult> {
    generate(conn, Some(date))
}

/// Load the latest persisted style profile, falling back to defaults if none
/// exists. Folds in feedback-learned verb overrides from the
/// `timesheet_feedback` table (P5-017).
pub fn load_style(conn: &Connection) -> AppResult<StyleProfile> {
    let mut profile: StyleProfile = match style_profile_repository::get_latest(conn)? {
        Some(row) => serde_json::from_str(&row.profile_json)
            .map_err(|e| crate::errors::AppError::Parse(format!("invalid style profile json: {e}")))?,
        None => StyleProfile::default(),
    };

    let feedbacks = timesheet_feedback_repository::get_all(conn)?;
    for fb in feedbacks {
        if let (Some(ov), Some(ev)) = (fb.original_verb, fb.edited_verb) {
            if ov != ev {
                profile.learned_verb_overrides.entry(ov).or_insert(ev);
            }
        }
    }

    Ok(profile)
}

fn build_content_json(task_groups: &[crate::timesheet::models::TaskGroupDraft]) -> String {
    let json = crate::timesheet::models::TimesheetDraftJson {
        task_groups: task_groups
            .iter()
            .map(|g| crate::timesheet::models::TaskGroupJson {
                title: g.title.clone(),
                ticket_reference: g.ticket_reference.clone(),
                bullets: g.bullets.iter().map(|b| b.text.clone()).collect(),
            })
            .collect(),
    };
    serde_json::to_string(&json).unwrap_or_default()
}

/// Update the stored style profile from uploaded example files (P5-007/008).
/// Returns the updated profile as JSON.
pub fn update_style_from_files(conn: &Connection, file_paths: &[String]) -> AppResult<StyleProfile> {
    let mut combined_text = String::new();
    let mut source_files: Vec<String> = Vec::new();

    for path_str in file_paths {
        let path = std::path::Path::new(path_str);
        match crate::timesheet::style_file_parser::extract_text(path) {
            Ok(text) => {
                combined_text.push_str(&text);
                combined_text.push('\n');
                source_files.push(path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            }
            Err(e) => log::warn!(target: "style", "Skipping file {:?}: {}", path, e),
        }
    }

    let new_profile = crate::timesheet::style_engine::analyze(&combined_text, source_files.clone());
    let profile_json = serde_json::to_string(&new_profile).map_err(|e| crate::errors::AppError::Parse(e.to_string()))?;
    let sources_json = serde_json::to_string(&source_files).ok();

    let user_id = user_repository::get(conn)?.map(|u| u.id).unwrap_or_else(|| "default".to_string());
    style_profile_repository::upsert(conn, &user_id, &profile_json, sources_json.as_deref())?;

    Ok(new_profile)
}

/// Feature-level groupings derived from current task groups for the dashboard
/// "Features" widget (P5-005) — computed on demand, not persisted.
pub fn get_features_for_date(conn: &Connection, date: &str) -> AppResult<Vec<feature_grouper::FeatureGroup>> {
    let task_groups_db = task_group_repository::get_by_date(conn, date)?;
    let style = load_style(conn)?;

    let draft_groups: Vec<crate::timesheet::models::TaskGroupDraft> = task_groups_db
        .iter()
        .map(|tg| crate::timesheet::models::TaskGroupDraft {
            title: tg.title.clone(),
            task_key: tg.task_key.clone(),
            ticket_reference: tg.ticket_reference.clone(),
            project_id: tg.project_id.clone(),
            bullets: vec![],
        })
        .collect();

    drop(style);
    Ok(feature_grouper::group(&draft_groups))
}
