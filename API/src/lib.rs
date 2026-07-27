pub mod commands;
pub mod errors;
pub mod git;
pub mod integrations;
pub mod models;
pub mod parsers;
pub mod repositories;
pub mod scanners;
pub mod services;
pub mod timesheet;
pub mod utils;

use std::sync::Mutex;
use rusqlite::Connection;
use tauri::Manager;
use utils::database;

/// Shared database connection wrapped in a Mutex for thread-safe access from commands.
pub struct DbState(pub Mutex<Connection>);

impl DbState {
    pub fn lock_db(&self) -> errors::AppResult<std::sync::MutexGuard<'_, Connection>> {
        self.0.lock().map_err(|_| errors::AppError::Unexpected("database lock poisoned".into()))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let conn = database::open_connection().expect("Failed to open database");
            database::run_migrations(&conn).expect("Failed to run database migrations");
            app.manage(DbState(Mutex::new(conn)));
            log::info!("AI Timesheet Agent started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Logging
            commands::log_command::log_message,
            // Settings
            commands::settings_commands::get_settings,
            commands::settings_commands::upsert_setting,
            // User
            commands::user_commands::get_user,
            commands::user_commands::save_user,
            // AI Tools
            commands::ai_tool_commands::get_ai_tools,
            commands::ai_tool_commands::update_ai_tool,
            // Projects
            commands::project_commands::discover_projects,
            commands::project_commands::get_projects,
            commands::project_commands::set_project_selection,
            // Onboarding
            commands::onboarding_commands::get_onboarding_state,
            commands::onboarding_commands::complete_onboarding,
            // Sessions
            commands::session_commands::scan_sessions,
            commands::session_commands::get_sessions,
            commands::session_commands::get_activities,
            commands::session_commands::get_timeline,
            commands::session_commands::get_session_dashboard,
            // Timesheet Intelligence
            commands::timesheet_commands::generate_timesheet,
            commands::timesheet_commands::regenerate_timesheet,
            commands::timesheet_commands::get_timesheet_by_date,
            commands::timesheet_commands::get_timesheet,
            commands::timesheet_commands::get_timesheet_history,
            commands::timesheet_commands::update_timesheet_content,
            commands::timesheet_commands::finalize_timesheet,
            commands::timesheet_commands::upload_style_examples,
            commands::timesheet_commands::get_style_profile,
            commands::timesheet_commands::submit_bullet_feedback,
            commands::timesheet_commands::get_task_groups,
            commands::timesheet_commands::get_feature_groups,
            commands::timesheet_commands::render_timesheet,
            // Git Intelligence
            commands::git_commands::scan_git,
            commands::git_commands::get_repositories,
            commands::git_commands::get_commits,
            commands::git_commands::get_file_changes,
            commands::git_commands::get_git_dashboard,
            // Dialog
            commands::dialog_commands::pick_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
