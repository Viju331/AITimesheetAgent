-- Migration 004: Timesheet Intelligence Engine

CREATE TABLE IF NOT EXISTS task_groups (
    id               TEXT PRIMARY KEY,
    project_id       TEXT REFERENCES projects(id) ON DELETE SET NULL,
    group_date       TEXT NOT NULL,
    title            TEXT NOT NULL,
    task_key         TEXT NOT NULL,
    ticket_reference TEXT,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_task_groups_date ON task_groups(group_date);

ALTER TABLE activities ADD COLUMN task_group_id TEXT REFERENCES task_groups(id) ON DELETE SET NULL;

CREATE TABLE IF NOT EXISTS timesheet_feedback (
    id            TEXT PRIMARY KEY,
    timesheet_id  TEXT NOT NULL REFERENCES generated_timesheets(id) ON DELETE CASCADE,
    original_text TEXT NOT NULL,
    edited_text   TEXT NOT NULL,
    original_verb TEXT,
    edited_verb   TEXT,
    created_at    TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_timesheet_feedback_timesheet ON timesheet_feedback(timesheet_id);
