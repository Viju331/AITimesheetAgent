use std::path::Path;
use super::command_runner::run_git;
use super::file_change_analyzer::categorize;
use super::models::{FileStatus, WorkingTreeChange};
use crate::errors::AppResult;

/// Inspect uncommitted work in a repository via `git status --porcelain` (P4-006).
pub fn analyze(repo_root: &Path) -> AppResult<Vec<WorkingTreeChange>> {
    let output = run_git(repo_root, &["status", "--porcelain=v1"])?;
    Ok(parse_porcelain(&output))
}

fn parse_porcelain(output: &str) -> Vec<WorkingTreeChange> {
    let mut changes = Vec::new();

    for line in output.lines() {
        if line.len() < 4 {
            continue;
        }
        let code = &line[0..2];
        let rest = &line[3..];

        // Renames are reported as "old -> new"; track the new path.
        let path = match rest.split_once(" -> ") {
            Some((_, new_path)) => new_path,
            None => rest,
        };

        let status = status_from_code(code);
        let category = categorize(path);
        changes.push(WorkingTreeChange { path: path.to_string(), status, category });
    }

    changes
}

fn status_from_code(code: &str) -> FileStatus {
    if code.contains('R') {
        FileStatus::Renamed
    } else if code.contains('A') || code == "??" {
        FileStatus::Added
    } else if code.contains('D') {
        FileStatus::Deleted
    } else if code.contains('C') {
        FileStatus::Copied
    } else {
        FileStatus::Modified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modified_and_untracked() {
        let output = " M src/app/foo.ts\n?? src/app/new-file.ts\n";
        let changes = parse_porcelain(output);
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].status, FileStatus::Modified);
        assert_eq!(changes[0].path, "src/app/foo.ts");
        assert_eq!(changes[1].status, FileStatus::Added);
    }

    #[test]
    fn parses_renames() {
        let output = "R  old.ts -> new.ts\n";
        let changes = parse_porcelain(output);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].status, FileStatus::Renamed);
        assert_eq!(changes[0].path, "new.ts");
    }

    #[test]
    fn parses_deleted() {
        let output = " D src/app/old.ts\n";
        let changes = parse_porcelain(output);
        assert_eq!(changes[0].status, FileStatus::Deleted);
    }

    #[test]
    fn ignores_blank_lines() {
        assert!(parse_porcelain("\n\n").is_empty());
    }
}
