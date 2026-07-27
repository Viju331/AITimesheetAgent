use std::path::Path;
use std::process::Command;
use crate::errors::{AppError, AppResult};

/// Run `git <args>` with `cwd` as the working directory and return stdout as
/// a UTF-8 string. Arguments are passed as a literal argv array — never
/// interpolated into a shell string — so there is no command-injection
/// surface even though commit messages, branch names, etc. are attacker-
/// controllable in theory.
pub fn run_git(cwd: &Path, args: &[&str]) -> AppResult<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| AppError::Git(format!("failed to spawn git in {:?}: {}", cwd, e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Git(format!(
            "git {} failed in {:?}: {}",
            args.join(" "),
            cwd,
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Same as [`run_git`] but returns `None` instead of erroring when the
/// command fails — used for best-effort lookups (e.g. reflog on a shallow
/// clone) where failure is an expected, non-fatal outcome.
pub fn run_git_lenient(cwd: &Path, args: &[&str]) -> Option<String> {
    run_git(cwd, args).ok()
}

/// True when `path` is (or is inside) a git working tree.
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}
