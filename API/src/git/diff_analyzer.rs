use std::collections::HashMap;
use std::path::Path;
use super::command_runner::run_git;
use super::file_change_analyzer::categorize;
use super::models::{FileChangeInfo, FileStatus};
use crate::errors::AppResult;

/// Quantify the files touched by a single commit: status (added/modified/...)
/// from `git diff-tree --name-status`, merged with per-file insertion/deletion
/// counts from `git show --numstat` (P4-008).
pub fn analyze_commit(repo_root: &Path, commit_hash: &str) -> AppResult<Vec<FileChangeInfo>> {
    let name_status =
        run_git(repo_root, &["diff-tree", "--no-commit-id", "-r", "--name-status", commit_hash])?;
    let numstat = run_git(repo_root, &["show", "--numstat", "--format=", commit_hash])?;

    let stats = parse_numstat(&numstat);
    Ok(parse_name_status(&name_status, &stats))
}

/// Parse `git show --numstat` output into a path -> (insertions, deletions) map.
/// Binary files report `-` for both counts, which we treat as zero.
fn parse_numstat(output: &str) -> HashMap<String, (i64, i64)> {
    let mut stats = HashMap::new();
    for line in output.lines() {
        let mut parts = line.splitn(3, '\t');
        let Some(added) = parts.next() else { continue };
        let Some(removed) = parts.next() else { continue };
        let Some(path) = parts.next() else { continue };

        let added: i64 = added.parse().unwrap_or(0);
        let removed: i64 = removed.parse().unwrap_or(0);
        stats.insert(path.to_string(), (added, removed));
    }
    stats
}

fn parse_name_status(output: &str, stats: &HashMap<String, (i64, i64)>) -> Vec<FileChangeInfo> {
    let mut files = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split('\t');
        let Some(code) = parts.next() else { continue };
        let Some(first_path) = parts.next() else { continue };

        // Renames/copies emit a similarity score (R100, C75) followed by old + new paths.
        let path = if code.starts_with('R') || code.starts_with('C') {
            parts.next().unwrap_or(first_path).to_string()
        } else {
            first_path.to_string()
        };

        let status = match code.chars().next().unwrap_or('M') {
            'A' => FileStatus::Added,
            'D' => FileStatus::Deleted,
            'R' => FileStatus::Renamed,
            'C' => FileStatus::Copied,
            _ => FileStatus::Modified,
        };

        let (insertions, deletions) = stats.get(&path).copied().unwrap_or((0, 0));
        let category = categorize(&path);

        files.push(FileChangeInfo { path, status, category, insertions, deletions });
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_status_and_numstat() {
        let name_status = "M\tsrc/app/foo.ts\nA\tsrc/app/foo.spec.ts\n";
        let numstat = "5\t2\tsrc/app/foo.ts\n10\t0\tsrc/app/foo.spec.ts\n";
        let stats = parse_numstat(numstat);
        let files = parse_name_status(name_status, &stats);

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].status, FileStatus::Modified);
        assert_eq!(files[0].insertions, 5);
        assert_eq!(files[0].deletions, 2);
        assert_eq!(files[1].status, FileStatus::Added);
        assert_eq!(files[1].insertions, 10);
    }

    #[test]
    fn handles_renames_with_similarity_score() {
        let name_status = "R100\told.ts\tnew.ts\n";
        let stats: HashMap<String, (i64, i64)> = HashMap::new();
        let files = parse_name_status(name_status, &stats);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, FileStatus::Renamed);
        assert_eq!(files[0].path, "new.ts");
    }

    #[test]
    fn handles_binary_files_as_zero_stats() {
        let numstat = "-\t-\tsrc/assets/logo.png\n";
        let stats = parse_numstat(numstat);
        assert_eq!(stats.get("src/assets/logo.png"), Some(&(0, 0)));
    }
}
