use tauri::State;
use crate::DbState;
use crate::errors::AppResult;
use crate::models::{
    FeatureGroupDto, GenerateTimesheetResponseDto, QualityScoreDto, StyleProfileDto, TaskGroupDto, TimesheetDto,
};
use crate::parsers::timestamp_engine;
use crate::repositories::{task_group_repository, timesheet_feedback_repository, timesheet_repository};
use crate::services::timesheet_pipeline_service::{self, GenerateResult};
use crate::timesheet::feedback_engine;

fn quality_dto(q: crate::timesheet::models::QualityScore) -> QualityScoreDto {
    QualityScoreDto { completeness: q.completeness, confidence: q.confidence, duplication: q.duplication, coverage: q.coverage, overall: q.overall }
}

fn result_to_dto(r: GenerateResult) -> GenerateTimesheetResponseDto {
    GenerateTimesheetResponseDto {
        timesheet_id: r.timesheet_id,
        task_group_count: r.task_group_count,
        bullet_count: r.bullet_count,
        quality_score: quality_dto(r.quality_score),
        plain_text: r.plain_text,
        markdown: r.markdown,
        html: r.html,
        content_json: r.content_json,
    }
}

fn ts_to_dto(t: timesheet_repository::GeneratedTimesheet) -> TimesheetDto {
    TimesheetDto {
        id: t.id,
        user_id: t.user_id,
        generated_date: t.generated_date,
        content: t.content,
        raw_draft: t.raw_draft,
        status: t.status,
        activity_ids: t.activity_ids,
        created_at: t.created_at,
        exported_at: t.exported_at,
    }
}

/// Run the full generation pipeline for the given date (defaults to today).
#[tauri::command]
pub fn generate_timesheet(state: State<'_, DbState>, date: Option<String>) -> AppResult<GenerateTimesheetResponseDto> {
    let conn = state.lock_db()?;
    let result = timesheet_pipeline_service::generate(&conn, date.as_deref())?;
    Ok(result_to_dto(result))
}

/// Regenerate the draft for a specific date, preserving history by creating a new version.
#[tauri::command]
pub fn regenerate_timesheet(state: State<'_, DbState>, date: String) -> AppResult<GenerateTimesheetResponseDto> {
    let conn = state.lock_db()?;
    let result = timesheet_pipeline_service::regenerate(&conn, &date)?;
    Ok(result_to_dto(result))
}

/// Fetch the latest generated timesheet for the given date.
#[tauri::command]
pub fn get_timesheet_by_date(state: State<'_, DbState>, date: Option<String>) -> AppResult<Option<TimesheetDto>> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);
    let mut timesheets = timesheet_repository::get_by_date(&conn, &date)?;
    timesheets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(timesheets.into_iter().next().map(ts_to_dto))
}

/// Fetch a specific timesheet by ID.
#[tauri::command]
pub fn get_timesheet(state: State<'_, DbState>, id: String) -> AppResult<Option<TimesheetDto>> {
    let conn = state.lock_db()?;
    Ok(timesheet_repository::get_by_id(&conn, &id)?.map(ts_to_dto))
}

/// List recent timesheets for the history view (P5-014).
#[tauri::command]
pub fn get_timesheet_history(state: State<'_, DbState>, limit: Option<i64>) -> AppResult<Vec<TimesheetDto>> {
    let conn = state.lock_db()?;
    let timesheets = timesheet_repository::get_history(&conn, limit.unwrap_or(20))?;
    Ok(timesheets.into_iter().map(ts_to_dto).collect())
}

/// Persist a manual edit to the draft content (P5-012).
#[tauri::command]
pub fn update_timesheet_content(state: State<'_, DbState>, id: String, content: String) -> AppResult<()> {
    let conn = state.lock_db()?;
    timesheet_repository::update_content(&conn, &id, &content)
}

/// Mark a timesheet as finalized (status → "final").
#[tauri::command]
pub fn finalize_timesheet(state: State<'_, DbState>, id: String) -> AppResult<()> {
    let conn = state.lock_db()?;
    timesheet_repository::update_status(&conn, &id, "final")
}

/// Upload example timesheet files to update the style profile (P5-007/008).
#[tauri::command]
pub fn upload_style_examples(state: State<'_, DbState>, file_paths: Vec<String>) -> AppResult<StyleProfileDto> {
    let conn = state.lock_db()?;
    let profile = timesheet_pipeline_service::update_style_from_files(&conn, &file_paths)?;
    Ok(StyleProfileDto {
        preferred_verbs: profile.preferred_verbs,
        bullet_char: format!("{:?}", profile.bullet_char),
        group_by: format!("{:?}", profile.group_by),
        verbosity: format!("{:?}", profile.verbosity),
        sentence_format: format!("{:?}", profile.sentence_format),
        heading_style: format!("{:?}", profile.heading_style),
        examples: profile.examples,
        source_files: profile.source_files,
    })
}

/// Get the current style profile.
#[tauri::command]
pub fn get_style_profile(state: State<'_, DbState>) -> AppResult<StyleProfileDto> {
    let conn = state.lock_db()?;
    let profile = timesheet_pipeline_service::load_style(&conn)?;
    Ok(StyleProfileDto {
        preferred_verbs: profile.preferred_verbs,
        bullet_char: format!("{:?}", profile.bullet_char),
        group_by: format!("{:?}", profile.group_by),
        verbosity: format!("{:?}", profile.verbosity),
        sentence_format: format!("{:?}", profile.sentence_format),
        heading_style: format!("{:?}", profile.heading_style),
        examples: profile.examples,
        source_files: profile.source_files,
    })
}

/// Record a user's edit to a generated bullet and learn from it (P5-017).
#[tauri::command]
pub fn submit_bullet_feedback(
    state: State<'_, DbState>,
    timesheet_id: String,
    original_text: String,
    edited_text: String,
) -> AppResult<()> {
    let conn = state.lock_db()?;
    let change = feedback_engine::detect_verb_change(&original_text, &edited_text);
    timesheet_feedback_repository::insert(
        &conn,
        &timesheet_id,
        &original_text,
        &edited_text,
        change.original_verb.as_deref(),
        change.edited_verb.as_deref(),
    )?;
    Ok(())
}

/// Task groups for the current date (for the dashboard task group widget).
#[tauri::command]
pub fn get_task_groups(state: State<'_, DbState>, date: Option<String>) -> AppResult<Vec<TaskGroupDto>> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);
    let groups = task_group_repository::get_by_date(&conn, &date)?;
    Ok(groups.into_iter().map(|g| TaskGroupDto { id: g.id, project_id: g.project_id, group_date: g.group_date, title: g.title, task_key: g.task_key, ticket_reference: g.ticket_reference, created_at: g.created_at }).collect())
}

/// Feature-level groupings for the dashboard (P5-005).
#[tauri::command]
pub fn get_feature_groups(state: State<'_, DbState>, date: Option<String>) -> AppResult<Vec<FeatureGroupDto>> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);
    let features = timesheet_pipeline_service::get_features_for_date(&conn, &date)?;
    Ok(features.into_iter().map(|f| FeatureGroupDto { title: f.title, task_titles: f.task_titles }).collect())
}

/// Render the current draft in different formats for the preview panel (P5-013).
#[tauri::command]
pub fn render_timesheet(state: State<'_, DbState>, id: String, format: String) -> AppResult<String> {
    let conn = state.lock_db()?;
    let timesheet = timesheet_repository::get_by_id(&conn, &id)?
        .ok_or_else(|| crate::errors::AppError::NotFound(format!("timesheet '{id}' not found")))?;

    let draft: crate::timesheet::models::TimesheetDraftJson = serde_json::from_str(&timesheet.content)
        .map_err(|e| crate::errors::AppError::Parse(format!("invalid draft json: {e}")))?;

    let style = timesheet_pipeline_service::load_style(&conn)?;
    let task_groups: Vec<crate::timesheet::models::TaskGroupDraft> = draft
        .task_groups
        .into_iter()
        .map(|tg| crate::timesheet::models::TaskGroupDraft {
            title: tg.title,
            task_key: String::new(),
            ticket_reference: tg.ticket_reference,
            project_id: None,
            bullets: tg.bullets.iter().map(|b| crate::timesheet::models::BulletPoint { text: b.clone(), source_activity_ids: vec![], confidence_score: 0.0 }).collect(),
        })
        .collect();

    Ok(match format.to_lowercase().as_str() {
        "markdown" | "md" => crate::timesheet::renderer::to_markdown(&task_groups, &style),
        "html" => crate::timesheet::renderer::to_html(&task_groups),
        _ => crate::timesheet::renderer::to_plain_text(&task_groups, &style),
    })
}
