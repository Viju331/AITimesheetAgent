use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

impl FileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileStatus::Added => "added",
            FileStatus::Modified => "modified",
            FileStatus::Deleted => "deleted",
            FileStatus::Renamed => "renamed",
            FileStatus::Copied => "copied",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileCategory {
    Angular,
    DotNet,
    Sql,
    Documentation,
    Configuration,
    Tests,
    Other,
}

impl FileCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileCategory::Angular => "Angular",
            FileCategory::DotNet => "DotNet",
            FileCategory::Sql => "SQL",
            FileCategory::Documentation => "Documentation",
            FileCategory::Configuration => "Configuration",
            FileCategory::Tests => "Tests",
            FileCategory::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GitChangeType {
    Feature,
    BugFix,
    Refactor,
    Test,
    Docs,
    Chore,
}

impl GitChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GitChangeType::Feature => "Feature",
            GitChangeType::BugFix => "Bug Fix",
            GitChangeType::Refactor => "Refactor",
            GitChangeType::Test => "Unit Test",
            GitChangeType::Docs => "Documentation",
            GitChangeType::Chore => "Chore",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileChangeInfo {
    pub path: String,
    pub status: FileStatus,
    pub category: FileCategory,
    pub insertions: i64,
    pub deletions: i64,
}

impl FileChangeInfo {
    pub fn file_name(&self) -> String {
        self.path.rsplit(['/', '\\']).next().unwrap_or(&self.path).to_string()
    }

    pub fn extension(&self) -> Option<String> {
        self.file_name().rsplit_once('.').map(|(_, ext)| ext.to_string())
    }

    pub fn folder(&self) -> String {
        match self.path.rsplit_once(['/', '\\']) {
            Some((dir, _)) => dir.to_string(),
            None => String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    /// ISO 8601, author's local offset as reported by git (`%aI`).
    pub commit_date: String,
    pub subject: String,
    pub files: Vec<FileChangeInfo>,
}

impl CommitInfo {
    pub fn insertions(&self) -> i64 {
        self.files.iter().map(|f| f.insertions).sum()
    }

    pub fn deletions(&self) -> i64 {
        self.files.iter().map(|f| f.deletions).sum()
    }
}

#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub current_branch: Option<String>,
    pub last_checkout_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkingTreeChange {
    pub path: String,
    pub status: FileStatus,
    pub category: FileCategory,
}
