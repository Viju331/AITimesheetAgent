use tauri::State;
use crate::DbState;
use crate::errors::AppResult;
use crate::models::UserDto;
use crate::repositories::user_repository;

#[tauri::command]
pub fn get_user(state: State<'_, DbState>) -> AppResult<Option<UserDto>> {
    let conn = state.lock_db()?;
    user_repository::get(&conn)
}

#[tauri::command]
pub fn save_user(state: State<'_, DbState>, name: String, email: Option<String>) -> AppResult<UserDto> {
    let conn = state.lock_db()?;
    user_repository::upsert(&conn, &name, email.as_deref())
}
