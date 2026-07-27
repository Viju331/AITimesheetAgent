use std::path::Path;
use super::command_runner::is_git_repo;
use crate::repositories::project_repository::Project;

/// A discovered repository, already mapped to its owning project (P4-002/P4-003).
/// Repository discovery piggybacks on the project scanner from onboarding (Phase 2),
/// which already records `has_git` for each project root — the repository root is
/// simply the project path when that flag is set.
pub struct DiscoveredRepository<'a> {
    pub project: &'a Project,
    pub root_path: String,
}

/// Find git repositories among the given (already project-selected) list.
/// Re-verifies `.git` presence on disk in case the project moved or was
/// un-initialized since the last project scan.
pub fn discover(projects: &[Project]) -> Vec<DiscoveredRepository<'_>> {
    projects
        .iter()
        .filter(|p| p.has_git && is_git_repo(Path::new(&p.path)))
        .map(|p| DiscoveredRepository { project: p, root_path: p.path.clone() })
        .collect()
}
