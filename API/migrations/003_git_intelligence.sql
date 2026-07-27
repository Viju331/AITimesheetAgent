-- Migration 003: Git Intelligence Engine

CREATE TABLE IF NOT EXISTS repositories (
    id               TEXT PRIMARY KEY,
    project_id       TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    root_path        TEXT NOT NULL UNIQUE,
    current_branch   TEXT,
    last_checkout_at TEXT,
    last_scanned_at  TEXT,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_repositories_project ON repositories(project_id);

CREATE TABLE IF NOT EXISTS file_changes (
    id              TEXT PRIMARY KEY,
    commit_id       TEXT REFERENCES git_activities(id) ON DELETE CASCADE,
    repository_id   TEXT REFERENCES repositories(id) ON DELETE CASCADE,
    file_path       TEXT NOT NULL,
    file_name       TEXT NOT NULL,
    extension       TEXT,
    folder          TEXT,
    change_type     TEXT NOT NULL,
    category        TEXT NOT NULL,
    insertions      INTEGER NOT NULL DEFAULT 0,
    deletions       INTEGER NOT NULL DEFAULT 0,
    is_working_tree INTEGER NOT NULL DEFAULT 0,
    detected_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_file_changes_commit ON file_changes(commit_id);
CREATE INDEX IF NOT EXISTS idx_file_changes_repo   ON file_changes(repository_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_git_activities_commit_hash ON git_activities(project_id, commit_hash);
