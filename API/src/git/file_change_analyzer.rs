use super::models::FileCategory;

const TEST_PATTERNS: &[&str] =
    &[".spec.ts", ".test.ts", "_test.", "test.java", "tests.cs", "__tests__/", "/tests/", "/test/"];

const ANGULAR_MARKERS: &[&str] =
    &[".component.", ".service.", ".module.", ".directive.", ".pipe.", ".guard.", ".resolver."];

const DOTNET_EXTENSIONS: &[&str] = &["cs", "csproj", "sln", "razor"];

const DOC_PATTERNS: &[&str] = &["readme", "changelog", "/docs/", "license"];

const CONFIG_EXTENSIONS: &[&str] = &["json", "yml", "yaml", "toml", "config"];
const CONFIG_NAMES: &[&str] = &["dockerfile", ".env.example", ".gitignore", ".editorconfig"];

/// Categorize a changed file by extension and path conventions (P4-007).
/// Rule order matters: tests are checked before Angular/.NET so that
/// `foo.component.spec.ts` is classified as `Tests`, not `Angular`.
pub fn categorize(path: &str) -> FileCategory {
    let lower = path.to_lowercase();
    let extension = lower.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");

    if TEST_PATTERNS.iter().any(|p| lower.contains(p)) {
        return FileCategory::Tests;
    }
    if extension == "sql" {
        return FileCategory::Sql;
    }
    if DOTNET_EXTENSIONS.contains(&extension) {
        return FileCategory::DotNet;
    }
    if ANGULAR_MARKERS.iter().any(|m| lower.contains(m))
        || lower.ends_with("angular.json")
        || (matches!(extension, "ts" | "html" | "scss" | "css") && lower.contains("/app/"))
    {
        return FileCategory::Angular;
    }
    if extension == "md" || DOC_PATTERNS.iter().any(|p| lower.contains(p)) {
        return FileCategory::Documentation;
    }
    if CONFIG_EXTENSIONS.contains(&extension) || CONFIG_NAMES.iter().any(|n| lower.ends_with(n)) {
        return FileCategory::Configuration;
    }

    FileCategory::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_test_files_before_angular() {
        assert_eq!(categorize("src/app/foo.component.spec.ts"), FileCategory::Tests);
    }

    #[test]
    fn classifies_angular_component() {
        assert_eq!(categorize("src/app/foo.component.ts"), FileCategory::Angular);
    }

    #[test]
    fn classifies_dotnet_files() {
        assert_eq!(categorize("Api/Controllers/UserController.cs"), FileCategory::DotNet);
    }

    #[test]
    fn classifies_sql_files() {
        assert_eq!(categorize("db/migrations/001_init.sql"), FileCategory::Sql);
    }

    #[test]
    fn classifies_documentation() {
        assert_eq!(categorize("README.md"), FileCategory::Documentation);
    }

    #[test]
    fn classifies_configuration() {
        assert_eq!(categorize("package.json"), FileCategory::Configuration);
    }

    #[test]
    fn falls_back_to_other() {
        assert_eq!(categorize("some/random/file.xyz"), FileCategory::Other);
    }
}
