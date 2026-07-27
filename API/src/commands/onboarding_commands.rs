use tauri::State;
use crate::DbState;
use crate::errors::AppResult;
use crate::models::OnboardingStateDto;
use crate::repositories::{ai_tool_repository, project_repository, settings_repository, user_repository};

#[tauri::command]
pub fn get_onboarding_state(state: State<'_, DbState>) -> AppResult<OnboardingStateDto> {
    let conn = state.lock_db()?;

    let has_user = user_repository::get(&conn)?.is_some();

    let tools = ai_tool_repository::get_all(&conn)?;
    let has_ai_tool = tools.iter().any(|t| t.is_enabled);

    let selected = project_repository::get_selected(&conn)?;
    let has_projects = !selected.is_empty();

    let is_complete = settings_repository::get_by_key(&conn, "onboarding.complete")?
        .map(|s| s.value == "true")
        .unwrap_or(false);

    Ok(OnboardingStateDto {
        has_user,
        has_ai_tool,
        has_projects,
        is_complete,
    })
}

#[tauri::command]
pub fn complete_onboarding(state: State<'_, DbState>) -> AppResult<()> {
    let conn = state.lock_db()?;
    settings_repository::upsert(&conn, "onboarding.complete", "true", false)?;
    Ok(())
}
