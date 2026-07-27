use tauri::State;
use crate::DbState;
use crate::errors::AppResult;
use crate::models::{CommitDto, FileChangeDto, GitDashboardDto, RepositoryDto};
use crate::parsers::timestamp_engine;
use crate::repositories::{file_change_repository, git_activity_repository, repository_repository};
use crate::services::git_scan_service::{self, GitScanSummary};

fn repository_to_dto(r: repository_repository::Repository) -> RepositoryDto {
    RepositoryDto {
        id: r.id,
        project_id: r.project_id,
        root_path: r.root_path,
        current_branch: r.current_branch,
        last_checkout_at: r.last_checkout_at,
        last_scanned_at: r.last_scanned_at,
        created_at: r.created_at,
    }
}

fn commit_to_dto(c: git_activity_repository::GitActivity) -> CommitDto {
    CommitDto {
        id: c.id,
        project_id: c.project_id,
        commit_hash: c.commit_hash,
        branch: c.branch,
        commit_date: c.commit_date,
        commit_message: c.commit_message,
        author_name: c.author_name,
        author_email: c.author_email,
        files_changed: c.files_changed,
        insertions: c.insertions,
        deletions: c.deletions,
        change_type: c.change_type,
        scanned_at: c.scanned_at,
    }
}

fn file_change_to_dto(f: file_change_repository::FileChange) -> FileChangeDto {
    FileChangeDto {
        id: f.id,
        commit_id: f.commit_id,
        repository_id: f.repository_id,
        file_path: f.file_path,
        file_name: f.file_name,
        extension: f.extension,
        folder: f.folder,
        change_type: f.change_type,
        category: f.category,
        insertions: f.insertions,
        deletions: f.deletions,
        is_working_tree: f.is_working_tree,
        detected_at: f.detected_at,
    }
}

/// Scan all selected, git-enabled projects for today's commits and
/// uncommitted work (P4-018 manual/auto refresh entry point).
#[tauri::command]
pub fn scan_git(state: State<'_, DbState>) -> AppResult<GitScanSummary> {
    let conn = state.lock_db()?;
    git_scan_service::scan_and_store(&conn)
}

#[tauri::command]
pub fn get_repositories(state: State<'_, DbState>) -> AppResult<Vec<RepositoryDto>> {
    let conn = state.lock_db()?;
    let repos = repository_repository::get_all(&conn)?;
    Ok(repos.into_iter().map(repository_to_dto).collect())
}

#[tauri::command]
pub fn get_commits(state: State<'_, DbState>, date: Option<String>) -> AppResult<Vec<CommitDto>> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);
    let commits = git_activity_repository::get_by_date(&conn, &date)?;
    Ok(commits.into_iter().map(commit_to_dto).collect())
}

#[tauri::command]
pub fn get_file_changes(
    state: State<'_, DbState>,
    repository_id: String,
    working_tree_only: bool,
) -> AppResult<Vec<FileChangeDto>> {
    let conn = state.lock_db()?;
    let changes = file_change_repository::get_by_repository(&conn, &repository_id, working_tree_only)?;
    Ok(changes.into_iter().map(file_change_to_dto).collect())
}

#[tauri::command]
pub fn get_git_dashboard(state: State<'_, DbState>, date: Option<String>) -> AppResult<GitDashboardDto> {
    let conn = state.lock_db()?;
    let date = date.unwrap_or_else(timestamp_engine::today_date_string);

    let repositories = repository_repository::get_all(&conn)?;
    let commits = git_activity_repository::get_by_date(&conn, &date)?;

    let mut modified_file_count = 0i64;
    for repo in &repositories {
        modified_file_count += file_change_repository::get_by_repository(&conn, &repo.id, true)?.len() as i64;
    }
    for commit in &commits {
        modified_file_count += commit.files_changed;
    }

    let activity_count = crate::repositories::activity_repository::get_by_date(&conn, &date)?
        .into_iter()
        .filter(|a| a.source_commit_id.is_some())
        .count() as i64;

    Ok(GitDashboardDto {
        repository_count: repositories.len() as i64,
        commit_count: commits.len() as i64,
        modified_file_count,
        activity_count,
        repositories: repositories.into_iter().map(repository_to_dto).collect(),
        recent_commits: commits.into_iter().map(commit_to_dto).collect(),
    })
}
