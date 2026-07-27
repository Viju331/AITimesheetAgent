use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use crate::errors::AppResult;
use crate::models::ProjectDto;

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "dist",
    "bin",
    "obj",
    ".angular",
    "target",
    ".git",
    ".svn",
    "__pycache__",
    ".pytest_cache",
    "build",
    "out",
    ".vs",
    "packages",
    "venv",
    ".venv",
    "bower_components",
    ".idea",
];

const MAX_DEPTH: usize = 6;

pub fn scan_folder(root_path: &str) -> AppResult<Vec<ProjectDto>> {
    let root = Path::new(root_path);
    if !root.is_dir() {
        return Ok(vec![]);
    }

    let mut results: Vec<ProjectDto> = Vec::new();
    let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
    queue.push_back((root.to_path_buf(), 0));

    while let Some((dir, depth)) = queue.pop_front() {
        if depth >= MAX_DEPTH {
            continue;
        }

        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let mut subdirs: Vec<PathBuf> = Vec::new();
        let mut has_angular_json = false;
        let mut has_sln = false;
        let mut has_csproj = false;
        let mut has_package_json = false;
        let mut has_git = false;

        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            let path = entry.path();

            if path.is_dir() {
                if name_str == ".git" {
                    has_git = true;
                } else if !SKIP_DIRS.contains(&&*name_str) {
                    subdirs.push(path);
                }
            } else if path.is_file() {
                match &*name_str {
                    "angular.json" => has_angular_json = true,
                    "package.json" => has_package_json = true,
                    _ => {
                        if name_str.ends_with(".sln") {
                            has_sln = true;
                        } else if name_str.ends_with(".csproj") {
                            has_csproj = true;
                        }
                    }
                }
            }
        }

        // Priority: Angular > DotNet > Node > Git (only at depth > 0)
        let tech_stack = if has_angular_json {
            Some("Angular")
        } else if has_sln || has_csproj {
            Some("DotNet")
        } else if has_package_json {
            Some("Node")
        } else if has_git && depth > 0 {
            Some("Git")
        } else {
            None
        };

        if let Some(stack) = tech_stack {
            let project_name = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| dir.to_string_lossy().to_string());

            let git_remote = if has_git { read_git_remote(&dir) } else { None };
            let now = chrono::Utc::now().to_rfc3339();

            results.push(ProjectDto {
                id: String::new(), // filled in after DB upsert
                name: project_name,
                path: dir.to_string_lossy().to_string(),
                tech_stack: Some(stack.to_string()),
                has_git,
                git_remote,
                is_selected: false,
                created_at: now.clone(),
                last_scanned: Some(now),
            });
        }

        // Always descend into subdirectories to find nested projects
        for sub in subdirs {
            queue.push_back((sub, depth + 1));
        }
    }

    Ok(results)
}

fn read_git_remote(dir: &Path) -> Option<String> {
    let config_path = dir.join(".git").join("config");
    let content = std::fs::read_to_string(config_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(url) = trimmed.strip_prefix("url = ") {
            return Some(url.to_string());
        }
    }
    None
}
