use std::path::Path;
use rusqlite::Connection;
use crate::errors::AppResult;
use crate::git::{branch_analyzer, commit_analyzer, repository_scanner, working_tree_analyzer};
use crate::parsers::timestamp_engine;
use crate::repositories::{
    activity_repository, file_change_repository, git_activity_repository, project_repository,
    repository_repository, user_repository,
};
use crate::repositories::file_change_repository::NewFileChange;
use crate::services::{activity_correlator, confidence_engine, git_classifier};

#[derive(Debug, Default, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitScanSummary {
    pub repositories_scanned: i64,
    pub commits_found: i64,
    pub working_tree_changes: i64,
    pub activities_correlated: i64,
    pub activities_created: i64,
}

/// Scan every selected, git-enabled project for today's commits and
/// uncommitted work, classify each commit, correlate it with existing
/// AI-sourced activities where the topics overlap, and persist everything
/// (repositories, commits, file changes, activities) (P4-001 orchestration).
pub fn scan_and_store(conn: &Connection) -> AppResult<GitScanSummary> {
    let mut summary = GitScanSummary::default();

    let selected_projects = project_repository::get_selected(conn)?;
    if selected_projects.is_empty() {
        return Ok(summary);
    }

    let user = user_repository::get(conn)?;
    let today = timestamp_engine::today_date_string();
    let today_activities = activity_repository::get_by_date(conn, &today)?;

    for discovered in repository_scanner::discover(&selected_projects) {
        summary.repositories_scanned += 1;
        let root = Path::new(&discovered.root_path);
        let project_id = &discovered.project.id;

        let branch_info = branch_analyzer::analyze(root).unwrap_or_else(|e| {
            log::warn!(target: "git_scan", "Branch analysis failed for {:?}: {}", root, e);
            crate::git::models::BranchInfo { current_branch: None, last_checkout_at: None }
        });

        let stored_repo = repository_repository::upsert(
            conn,
            project_id,
            &discovered.root_path,
            branch_info.current_branch.as_deref(),
            branch_info.last_checkout_at.as_deref(),
        )?;

        let author = user
            .as_ref()
            .and_then(|u| u.email.clone())
            .or_else(|| commit_analyzer::local_git_email(root));

        let commits = match commit_analyzer::analyze_today(root, author.as_deref()) {
            Ok(c) => c,
            Err(e) => {
                log::warn!(target: "git_scan", "Commit analysis failed for {:?}: {}", root, e);
                Vec::new()
            }
        };
        summary.commits_found += commits.len() as i64;

        let mut uncorrelated: Vec<activity_repository::Activity> = today_activities
            .iter()
            .filter(|a| a.project_id.as_deref() == Some(project_id.as_str()) && a.source_commit_id.is_none())
            .cloned()
            .collect();

        for commit in &commits {
            let change_type = git_classifier::classify(commit);

            let stored_commit = git_activity_repository::upsert(
                conn,
                project_id,
                &commit.hash,
                branch_info.current_branch.as_deref(),
                &commit.commit_date,
                &commit.subject,
                Some(&commit.author_name),
                Some(&commit.author_email),
                commit.files.len() as i64,
                commit.insertions(),
                commit.deletions(),
                Some(change_type.as_str()),
            )?;

            file_change_repository::delete_by_commit(conn, &stored_commit.id)?;
            for file in &commit.files {
                file_change_repository::insert(
                    conn,
                    Some(&stored_commit.id),
                    Some(&stored_repo.id),
                    &NewFileChange {
                        file_path: file.path.clone(),
                        extension: file.extension(),
                        folder: Some(file.folder()),
                        change_type: file.status.as_str().to_string(),
                        category: file.category.as_str().to_string(),
                        insertions: file.insertions,
                        deletions: file.deletions,
                    },
                    false,
                )?;
            }

            if let Some(matched) = activity_correlator::find_matching_activity(&commit.subject, &uncorrelated) {
                let confidence = confidence_engine::correlated_confidence(true, false, true);
                activity_repository::update_correlation(conn, &matched.id, Some(&stored_commit.id), confidence)?;
                summary.activities_correlated += 1;
                let matched_id = matched.id.clone();
                uncorrelated.retain(|a| a.id != matched_id);
            } else {
                let title = git_classifier::generate_title(change_type, &commit.subject);
                let activity_date: String = commit.commit_date.chars().take(10).collect();
                let confidence = confidence_engine::git_only_confidence(commit, branch_info.current_branch.as_deref());

                activity_repository::insert(
                    conn,
                    Some(project_id),
                    &activity_date,
                    change_type.as_str(),
                    &title,
                    Some(&commit.subject),
                    None,
                    Some(&stored_commit.id),
                    confidence,
                )?;
                summary.activities_created += 1;
            }
        }

        let working_tree = match working_tree_analyzer::analyze(root) {
            Ok(w) => w,
            Err(e) => {
                log::warn!(target: "git_scan", "Working tree analysis failed for {:?}: {}", root, e);
                Vec::new()
            }
        };
        summary.working_tree_changes += working_tree.len() as i64;

        let new_changes: Vec<NewFileChange> = working_tree
            .iter()
            .map(|w| NewFileChange {
                file_path: w.path.clone(),
                extension: w.path.rsplit_once('.').map(|(_, e)| e.to_string()),
                folder: w.path.rsplit_once(['/', '\\']).map(|(d, _)| d.to_string()),
                change_type: w.status.as_str().to_string(),
                category: w.category.as_str().to_string(),
                insertions: 0,
                deletions: 0,
            })
            .collect();
        file_change_repository::replace_working_tree(conn, &stored_repo.id, &new_changes)?;

        if !working_tree.is_empty() {
            for activity in &uncorrelated {
                let confidence = confidence_engine::correlated_confidence(true, true, false);
                activity_repository::update_correlation(conn, &activity.id, None, confidence)?;
            }
        }
    }

    Ok(summary)
}
