use tauri::State;
use crate::DbState;
use crate::errors::AppResult;
use crate::models::AiToolDto;
use crate::repositories::ai_tool_repository;

#[tauri::command]
pub fn get_ai_tools(state: State<'_, DbState>) -> AppResult<Vec<AiToolDto>> {
    let conn = state.lock_db()?;
    ai_tool_repository::get_all(&conn)
}

#[tauri::command]
pub fn update_ai_tool(
    state: State<'_, DbState>,
    id: String,
    is_enabled: bool,
    session_folder: Option<String>,
) -> AppResult<AiToolDto> {
    let conn = state.lock_db()?;
    ai_tool_repository::update(&conn, &id, is_enabled, session_folder.as_deref())
}
