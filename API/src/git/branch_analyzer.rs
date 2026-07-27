use std::path::Path;
use super::command_runner::{run_git, run_git_lenient};
use super::models::BranchInfo;
use crate::errors::AppResult;

/// Determine the current branch and an approximate "last checkout" timestamp (P4-004).
/// Last checkout is read from the reflog when available (most accurate); falls
/// back to the most recent commit's date on shallow clones or repos with no reflog.
pub fn analyze(repo_root: &Path) -> AppResult<BranchInfo> {
    let current_branch = run_git(repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "HEAD");

    let last_checkout_at = run_git_lenient(repo_root, &["log", "-g", "--format=%cI", "-1", "HEAD"])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            run_git_lenient(repo_root, &["log", "-1", "--format=%cI"]).map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty());

    Ok(BranchInfo { current_branch, last_checkout_at })
}
