use std::path::Path;
use super::command_runner::run_git;
use super::diff_analyzer;
use super::models::CommitInfo;
use crate::errors::AppResult;

const FIELD_SEP: &str = "\x1f";

/// List today's commits by the given author (email or name), with per-file
/// change details merged in via the diff analyzer (P4-005).
/// `author` may be an email address or a plain name — git's `--author` flag
/// matches against both the name and email fields of a commit.
pub fn analyze_today(repo_root: &Path, author: Option<&str>) -> AppResult<Vec<CommitInfo>> {
    let format_arg = format!("--format=%H{FIELD_SEP}%ae{FIELD_SEP}%an{FIELD_SEP}%aI{FIELD_SEP}%s");
    let author_arg = author.map(|a| format!("--author={a}"));

    let mut args: Vec<&str> = vec!["log", "--since=midnight", "--no-merges", &format_arg];
    if let Some(ref a) = author_arg {
        args.push(a);
    }

    let output = run_git(repo_root, &args)?;
    let mut commits = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.splitn(5, FIELD_SEP);
        let Some(hash) = parts.next() else { continue };
        let Some(author_email) = parts.next() else { continue };
        let Some(author_name) = parts.next() else { continue };
        let Some(commit_date) = parts.next() else { continue };
        let subject = parts.next().unwrap_or("").to_string();

        let files = diff_analyzer::analyze_commit(repo_root, hash).unwrap_or_default();

        commits.push(CommitInfo {
            hash: hash.to_string(),
            author_name: author_name.to_string(),
            author_email: author_email.to_string(),
            commit_date: commit_date.to_string(),
            subject,
            files,
        });
    }

    Ok(commits)
}

/// Resolve the local git identity (`git config user.email`) to use as an
/// author filter when the app's user profile has no email on file.
pub fn local_git_email(repo_root: &Path) -> Option<String> {
    run_git(repo_root, &["config", "user.email"]).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_log_lines_with_unit_separator() {
        let line = format!("abc123{FIELD_SEP}u@e.com{FIELD_SEP}User Name{FIELD_SEP}2026-06-15T09:00:00+10:00{FIELD_SEP}fix township dropdown");
        let mut parts = line.splitn(5, FIELD_SEP);
        assert_eq!(parts.next(), Some("abc123"));
        assert_eq!(parts.next(), Some("u@e.com"));
        assert_eq!(parts.next(), Some("User Name"));
        assert_eq!(parts.next(), Some("2026-06-15T09:00:00+10:00"));
        assert_eq!(parts.next(), Some("fix township dropdown"));
    }
}
