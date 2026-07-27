use serde::{Deserialize, Serialize};
use tauri::State;
use crate::DbState;
use crate::errors::AppResult;
use crate::repositories::settings_repository;

#[derive(Debug, Serialize)]
pub struct SettingDto {
    pub key: String,
    pub value: String,
    pub is_encrypted: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSettingRequest {
    pub key: String,
    pub value: String,
    pub is_encrypted: bool,
}

#[tauri::command]
pub fn get_settings(state: State<'_, DbState>) -> AppResult<Vec<SettingDto>> {
    let conn = state.lock_db()?;
    let settings = settings_repository::get_all(&conn)?;
    Ok(settings
        .into_iter()
        .map(|s| SettingDto {
            key: s.key,
            value: s.value,
            is_encrypted: s.is_encrypted,
        })
        .collect())
}

#[tauri::command]
pub fn upsert_setting(state: State<'_, DbState>, request: UpsertSettingRequest) -> AppResult<SettingDto> {
    let conn = state.lock_db()?;
    let s = settings_repository::upsert(&conn, &request.key, &request.value, request.is_encrypted)?;
    Ok(SettingDto {
        key: s.key,
        value: s.value,
        is_encrypted: s.is_encrypted,
    })
}
