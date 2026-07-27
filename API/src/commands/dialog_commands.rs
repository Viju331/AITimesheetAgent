use tauri_plugin_dialog::DialogExt;
use crate::errors::AppResult;

#[tauri::command]
pub fn pick_folder(app: tauri::AppHandle) -> AppResult<Option<String>> {
    let folder = app.dialog().file().blocking_pick_folder();
    Ok(folder.map(|f| f.to_string()))
}
