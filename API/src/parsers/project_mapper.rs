use crate::repositories::project_repository::Project;

/// Find the best-matching known project for a session's reported working
/// directory or workspace path. Exact path match is preferred; falls back to
/// the longest known project path that contains the session path as a
/// subdirectory (handles sessions opened on a sub-folder of a tracked repo).
pub fn match_project<'a>(session_path: &str, projects: &'a [Project]) -> Option<&'a Project> {
    let normalized = normalize_path(session_path);
    if normalized.is_empty() {
        return None;
    }

    if let Some(p) = projects.iter().find(|p| normalize_path(&p.path) == normalized) {
        return Some(p);
    }

    projects
        .iter()
        .filter(|p| {
            let project_path = normalize_path(&p.path);
            !project_path.is_empty() && normalized.starts_with(&project_path)
        })
        .max_by_key(|p| p.path.len())
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_project(path: &str) -> Project {
        Project {
            id: "p1".to_string(),
            name: "test".to_string(),
            path: path.to_string(),
            tech_stack: None,
            has_git: true,
            git_remote: None,
            is_selected: true,
            created_at: Utc::now().to_rfc3339(),
            last_scanned: None,
        }
    }

    #[test]
    fn matches_exact_path_case_and_slash_insensitive() {
        let projects = vec![make_project("D:\\Code\\MyApp")];
        let result = match_project("d:/code/myapp", &projects);
        assert_eq!(result.unwrap().path, "D:\\Code\\MyApp");
    }

    #[test]
    fn matches_subdirectory_to_parent_project() {
        let projects = vec![make_project("D:\\Code\\MyApp")];
        let result = match_project("D:\\Code\\MyApp\\src\\components", &projects);
        assert_eq!(result.unwrap().path, "D:\\Code\\MyApp");
    }

    #[test]
    fn picks_longest_match_among_nested_projects() {
        let projects = vec![make_project("D:\\Code"), make_project("D:\\Code\\MyApp")];
        let result = match_project("D:\\Code\\MyApp\\src", &projects);
        assert_eq!(result.unwrap().path, "D:\\Code\\MyApp");
    }

    #[test]
    fn returns_none_when_no_match() {
        let projects = vec![make_project("D:\\Code\\MyApp")];
        assert!(match_project("E:\\Other", &projects).is_none());
    }
}
