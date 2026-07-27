use tauri::command;

#[command]
pub fn log_message(level: String, message: String, context: String) {
    let tag = if context.is_empty() {
        "frontend".to_string()
    } else {
        format!("frontend::{context}")
    };

    match level.as_str() {
        "debug" => log::debug!(target: &tag, "{}", message),
        "info"  => log::info! (target: &tag, "{}", message),
        "warn"  => log::warn! (target: &tag, "{}", message),
        "error" => log::error!(target: &tag, "{}", message),
        _       => log::info! (target: &tag, "{}", message),
    }
}
