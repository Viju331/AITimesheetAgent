-- Migration: 001-initial-schema
-- Phase: 0 / Foundation
-- Applied by: Tauri startup via tauri-plugin-sql

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ============================================================
-- users
-- Single-user app; row created on first onboarding completion.
-- ============================================================
CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY,               -- UUID v4
    name        TEXT NOT NULL,
    email       TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ============================================================
-- settings
-- Key/value store for app-level and user-level config.
-- Sensitive values (API keys) are stored encrypted; the value
-- column holds ciphertext and is_encrypted flags the row.
-- ============================================================
CREATE TABLE IF NOT EXISTS settings (
    id            TEXT PRIMARY KEY,
    key           TEXT NOT NULL UNIQUE,
    value         TEXT NOT NULL,
    is_encrypted  INTEGER NOT NULL DEFAULT 0,   -- 1 = value is encrypted
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ============================================================
-- ai_tools
-- Registry of AI coding assistants the user has enabled.
-- ============================================================
CREATE TABLE IF NOT EXISTS ai_tools (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,       -- 'claude_code' | 'cursor' | 'codex'
    display_name    TEXT NOT NULL,
    session_folder  TEXT,                       -- resolved absolute path
    is_enabled      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ============================================================
-- projects
-- Developer projects discovered from configured root folders.
-- ============================================================
CREATE TABLE IF NOT EXISTS projects (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    path          TEXT NOT NULL UNIQUE,
    tech_stack    TEXT,                         -- 'angular' | 'dotnet' | 'node' | 'java' | 'python' | 'go' | 'unknown'
    has_git       INTEGER NOT NULL DEFAULT 0,
    git_remote    TEXT,
    is_selected   INTEGER NOT NULL DEFAULT 0,  -- user has included this project
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    last_scanned  TEXT
);

CREATE INDEX IF NOT EXISTS idx_projects_is_selected ON projects (is_selected);

-- ============================================================
-- ai_sessions
-- One row per parsed AI tool session file (daily boundary).
-- ============================================================
CREATE TABLE IF NOT EXISTS ai_sessions (
    id            TEXT PRIMARY KEY,
    project_id    TEXT REFERENCES projects(id) ON DELETE SET NULL,
    tool_id       TEXT REFERENCES ai_tools(id) ON DELETE SET NULL,
    session_date  TEXT NOT NULL,               -- ISO date: '2025-06-15'
    file_path     TEXT NOT NULL,
    raw_summary   TEXT,                        -- extracted text from session
    parsed_at     TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_ai_sessions_session_date ON ai_sessions (session_date);
CREATE INDEX IF NOT EXISTS idx_ai_sessions_project_id  ON ai_sessions (project_id);

-- ============================================================
-- git_activities
-- One row per commit captured during a git scan.
-- ============================================================
CREATE TABLE IF NOT EXISTS git_activities (
    id              TEXT PRIMARY KEY,
    project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    commit_hash     TEXT NOT NULL,
    branch          TEXT,
    commit_date     TEXT NOT NULL,
    commit_message  TEXT NOT NULL,
    author_name     TEXT,
    author_email    TEXT,
    files_changed   INTEGER NOT NULL DEFAULT 0,
    insertions      INTEGER NOT NULL DEFAULT 0,
    deletions       INTEGER NOT NULL DEFAULT 0,
    change_type     TEXT,                      -- 'feature' | 'bugfix' | 'refactor' | 'test' | 'docs' | 'chore'
    scanned_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_git_activities_project_id  ON git_activities (project_id);
CREATE INDEX IF NOT EXISTS idx_git_activities_commit_date ON git_activities (commit_date);

-- ============================================================
-- activities
-- Normalized, deduplicated work items produced by the Activity
-- Engine from ai_sessions + git_activities.
-- ============================================================
CREATE TABLE IF NOT EXISTS activities (
    id                TEXT PRIMARY KEY,
    project_id        TEXT REFERENCES projects(id) ON DELETE SET NULL,
    activity_date     TEXT NOT NULL,           -- ISO date: '2025-06-15'
    activity_type     TEXT NOT NULL,           -- 'feature' | 'bugfix' | 'refactor' | 'test' | 'docs' | 'chore' | 'meeting'
    title             TEXT NOT NULL,
    description       TEXT,
    source_session_id TEXT REFERENCES ai_sessions(id) ON DELETE SET NULL,
    source_commit_id  TEXT REFERENCES git_activities(id) ON DELETE SET NULL,
    confidence_score  REAL NOT NULL DEFAULT 0.0,  -- 0.0–1.0
    is_flagged        INTEGER NOT NULL DEFAULT 0,  -- low confidence, needs review
    created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_activities_activity_date ON activities (activity_date);
CREATE INDEX IF NOT EXISTS idx_activities_project_id   ON activities (project_id);
CREATE INDEX IF NOT EXISTS idx_activities_is_flagged   ON activities (is_flagged);

-- ============================================================
-- style_profiles
-- Writing style learned from user's example timesheets.
-- ============================================================
CREATE TABLE IF NOT EXISTS style_profiles (
    id            TEXT PRIMARY KEY,
    user_id       TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    profile_json  TEXT NOT NULL,               -- StyleProfile JSON (see docs/research/style-profile-model.md)
    source_files  TEXT,                        -- JSON array of uploaded filenames
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- ============================================================
-- generated_timesheets
-- AI-generated draft timesheets, editable before export.
-- ============================================================
CREATE TABLE IF NOT EXISTS generated_timesheets (
    id              TEXT PRIMARY KEY,
    user_id         TEXT REFERENCES users(id) ON DELETE SET NULL,
    generated_date  TEXT NOT NULL,             -- ISO date the timesheet covers
    content         TEXT NOT NULL,             -- final Markdown content (after user edits)
    raw_draft       TEXT,                      -- original AI-generated draft before edits
    status          TEXT NOT NULL DEFAULT 'draft',  -- 'draft' | 'approved' | 'exported'
    activity_ids    TEXT,                      -- JSON array of activity IDs that sourced this
    ai_provider     TEXT,                      -- 'openai' | 'azure_openai'
    ai_model        TEXT,                      -- e.g. 'gpt-4o'
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    exported_at     TEXT
);

CREATE INDEX IF NOT EXISTS idx_generated_timesheets_generated_date ON generated_timesheets (generated_date);
CREATE INDEX IF NOT EXISTS idx_generated_timesheets_status         ON generated_timesheets (status);
