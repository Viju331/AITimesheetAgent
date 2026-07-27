-- Migration 005: Work Management & Communication Integrations

CREATE TABLE IF NOT EXISTS integration_accounts (
    id               TEXT PRIMARY KEY,
    provider         TEXT NOT NULL,
    display_name     TEXT NOT NULL,
    account_id       TEXT,
    base_url         TEXT,
    access_token     TEXT,
    refresh_token    TEXT,
    expires_at       TEXT,
    scopes           TEXT,
    status           TEXT NOT NULL DEFAULT 'active',
    error_message    TEXT,
    last_synced_at   TEXT,
    created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_integration_accounts_provider ON integration_accounts(provider);

CREATE TABLE IF NOT EXISTS work_items (
    id               TEXT PRIMARY KEY,
    account_id       TEXT NOT NULL REFERENCES integration_accounts(id) ON DELETE CASCADE,
    provider         TEXT NOT NULL,
    provider_item_id TEXT NOT NULL,
    title            TEXT NOT NULL,
    item_type        TEXT,
    status           TEXT,
    priority         TEXT,
    project_key      TEXT,
    project_name     TEXT,
    assigned_to      TEXT,
    due_date         TEXT,
    url              TEXT,
    extra_json       TEXT,
    synced_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_work_items_provider_key ON work_items(account_id, provider_item_id);
CREATE INDEX  IF NOT EXISTS idx_work_items_account ON work_items(account_id);

CREATE TABLE IF NOT EXISTS calendar_events (
    id                TEXT PRIMARY KEY,
    account_id        TEXT NOT NULL REFERENCES integration_accounts(id) ON DELETE CASCADE,
    provider_event_id TEXT NOT NULL,
    title             TEXT NOT NULL,
    event_type        TEXT NOT NULL DEFAULT 'meeting',
    start_time        TEXT NOT NULL,
    end_time          TEXT NOT NULL,
    event_date        TEXT NOT NULL,
    duration_minutes  INTEGER NOT NULL DEFAULT 0,
    attendees_json    TEXT,
    is_organizer      INTEGER NOT NULL DEFAULT 0,
    synced_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_calendar_events_provider ON calendar_events(account_id, provider_event_id);
CREATE INDEX  IF NOT EXISTS idx_calendar_events_date ON calendar_events(event_date);

CREATE TABLE IF NOT EXISTS communication_signals (
    id               TEXT PRIMARY KEY,
    account_id       TEXT NOT NULL REFERENCES integration_accounts(id) ON DELETE CASCADE,
    source           TEXT NOT NULL,
    signal_type      TEXT NOT NULL,
    content          TEXT NOT NULL,
    ticket_reference TEXT,
    signal_date      TEXT NOT NULL,
    confidence       REAL NOT NULL DEFAULT 0.5,
    synced_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);

CREATE INDEX IF NOT EXISTS idx_communication_signals_date ON communication_signals(signal_date);
