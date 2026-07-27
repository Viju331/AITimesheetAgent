-- Migration 001: Initial Schema
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    email      TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE IF NOT EXISTS settings (
    id           TEXT PRIMARY KEY,
    key          TEXT NOT NULL UNIQUE,
    value        TEXT NOT NULL,
    is_encrypted INTEGER NOT NULL DEFAULT 0,
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE IF NOT EXISTS ai_tools (
    id             TEXT PRIMARY KEY,
    name           TEXT NOT NULL UNIQUE,
    display_name   TEXT NOT NULL,
    session_folder TEXT,
    is_enabled     INTEGER NOT NULL DEFAULT 0,
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE IF NOT EXISTS projects (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    path         TEXT NOT NULL UNIQUE,
    tech_stack   TEXT,
    has_git      INTEGER NOT NULL DEFAULT 0,
    git_remote   TEXT,
    is_selected  INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    last_scanned TEXT
);

CREATE INDEX IF NOT EXISTS idx_projects_is_selected ON projects(is_selected);

CREATE TABLE IF NOT EXISTS ai_sessions (
    id           TEXT PRIMARY KEY,
    project_id   TEXT REFERENCES projects(id) ON DELETE SET NULL,
    tool_id      TEXT REFERENCES ai_tools(id) ON DELETE SET NULL,
    session_date TEXT NOT NULL,
    file_path    TEXT NOT NULL,
    raw_summary  TEXT,
    parsed_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_ai_sessions_date       ON ai_sessions(session_date);
CREATE INDEX IF NOT EXISTS idx_ai_sessions_project_id ON ai_sessions(project_id);

CREATE TABLE IF NOT EXISTS git_activities (
    id             TEXT PRIMARY KEY,
    project_id     TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    commit_hash    TEXT NOT NULL,
    branch         TEXT,
    commit_date    TEXT NOT NULL,
    commit_message TEXT NOT NULL,
    author_name    TEXT,
    author_email   TEXT,
    files_changed  INTEGER NOT NULL DEFAULT 0,
    insertions     INTEGER NOT NULL DEFAULT 0,
    deletions      INTEGER NOT NULL DEFAULT 0,
    change_type    TEXT,
    scanned_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_git_activities_project_id  ON git_activities(project_id);
CREATE INDEX IF NOT EXISTS idx_git_activities_commit_date ON git_activities(commit_date);

CREATE TABLE IF NOT EXISTS activities (
    id               TEXT PRIMARY KEY,
    project_id       TEXT REFERENCES projects(id) ON DELETE SET NULL,
    activity_date    TEXT NOT NULL,
    activity_type    TEXT NOT NULL,
    title            TEXT NOT NULL,
    description      TEXT,
    source_session_id TEXT REFERENCES ai_sessions(id) ON DELETE SET NULL,
    source_commit_id  TEXT REFERENCES git_activities(id) ON DELETE SET NULL,
    confidence_score REAL NOT NULL DEFAULT 0.0,
    is_flagged       INTEGER NOT NULL DEFAULT 0,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_activities_date       ON activities(activity_date);
CREATE INDEX IF NOT EXISTS idx_activities_project_id ON activities(project_id);
CREATE INDEX IF NOT EXISTS idx_activities_is_flagged ON activities(is_flagged);

CREATE TABLE IF NOT EXISTS style_profiles (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    profile_json TEXT NOT NULL,
    source_files TEXT,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE TABLE IF NOT EXISTS generated_timesheets (
    id             TEXT PRIMARY KEY,
    user_id        TEXT REFERENCES users(id) ON DELETE SET NULL,
    generated_date TEXT NOT NULL,
    content        TEXT NOT NULL,
    raw_draft      TEXT,
    status         TEXT NOT NULL DEFAULT 'draft',
    activity_ids   TEXT,
    ai_provider    TEXT,
    ai_model       TEXT,
    created_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    exported_at    TEXT
);

CREATE INDEX IF NOT EXISTS idx_timesheets_date   ON generated_timesheets(generated_date);
CREATE INDEX IF NOT EXISTS idx_timesheets_status ON generated_timesheets(status);

-- Seed default AI tools
INSERT OR IGNORE INTO ai_tools (id, name, display_name, is_enabled) VALUES
  ('tool-claude', 'claude_code', 'Claude Code', 0),
  ('tool-cursor', 'cursor',      'Cursor',      0),
  ('tool-codex',  'codex',       'Codex CLI',   0);
