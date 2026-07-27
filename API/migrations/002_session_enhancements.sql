-- Migration 002: AI Session Parsing Enhancements

ALTER TABLE ai_sessions ADD COLUMN message_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE ai_sessions ADD COLUMN started_at TEXT;
ALTER TABLE ai_sessions ADD COLUMN ended_at TEXT;
ALTER TABLE ai_sessions ADD COLUMN file_mtime TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_ai_sessions_file_path ON ai_sessions(file_path);

CREATE TABLE IF NOT EXISTS timeline_events (
    id          TEXT PRIMARY KEY,
    project_id  TEXT REFERENCES projects(id) ON DELETE SET NULL,
    session_id  TEXT REFERENCES ai_sessions(id) ON DELETE CASCADE,
    activity_id TEXT REFERENCES activities(id) ON DELETE CASCADE,
    event_date  TEXT NOT NULL,
    event_time  TEXT NOT NULL,
    title       TEXT NOT NULL,
    description TEXT,
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_timeline_events_date    ON timeline_events(event_date);
CREATE INDEX IF NOT EXISTS idx_timeline_events_project ON timeline_events(project_id);
