use tauri::State;
use crate::DbState;
use crate::errors::AppResult;
use crate::models::ProjectDto;
use crate::repositories::project_repository;
use crate::scanners::project_scanner;

fn project_to_dto(p: project_repository::Project) -> ProjectDto {
    ProjectDto {
        id: p.id,
        name: p.name,
        path: p.path,
        tech_stack: p.tech_stack,
        has_git: p.has_git,
        git_remote: p.git_remote,
        is_selected: p.is_selected,
        created_at: p.created_at,
        last_scanned: p.last_scanned,
    }
}

/// Scan one or more root folders, upsert discovered projects into the DB,
/// and return the full up-to-date project list.
#[tauri::command]
pub fn discover_projects(
    state: State<'_, DbState>,
    root_paths: Vec<String>,
) -> AppResult<Vec<ProjectDto>> {
    let conn = state.lock_db()?;

    for root_path in &root_paths {
        let discovered = project_scanner::scan_folder(root_path)?;
        for p in discovered {
            project_repository::upsert(
                &conn,
                &p.name,
                &p.path,
                p.tech_stack.as_deref(),
                p.has_git,
                p.git_remote.as_deref(),
            )?;
        }
    }

    let all = project_repository::get_all(&conn)?;
    Ok(all.into_iter().map(project_to_dto).collect())
}

#[tauri::command]
pub fn get_projects(state: State<'_, DbState>) -> AppResult<Vec<ProjectDto>> {
    let conn = state.lock_db()?;
    let projects = project_repository::get_all(&conn)?;
    Ok(projects.into_iter().map(project_to_dto).collect())
}

#[tauri::command]
pub fn set_project_selection(
    state: State<'_, DbState>,
    id: String,
    is_selected: bool,
) -> AppResult<()> {
    let conn = state.lock_db()?;
    project_repository::update_selection(&conn, &id, is_selected)
}
