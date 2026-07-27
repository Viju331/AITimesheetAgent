-- AI Timesheet Intelligence Platform — Master Schema Reference
-- This file is the annotated reference. Actual migrations are in migrations/.
-- Do not run this file directly; use 001-initial-schema.sql.

-- Table relationships:
--
--   users ─────────────────────┬── style_profiles
--                              └── generated_timesheets
--
--   ai_tools ──────────────────── ai_sessions
--
--   projects ──────────────────┬── ai_sessions
--                              ├── git_activities
--                              └── activities
--
--   activities ─────────────────┬── (source) ai_sessions
--                               └── (source) git_activities

-- Entity: User
--   id            UUID
--   name          display name
--   email         optional, used for git author matching
--   created_at    ISO 8601 UTC

-- Entity: Setting
--   key           dot-notation key (e.g. 'app.theme', 'ai.openai_api_key')
--   value         string value; if is_encrypted=1, value is base64(AES-256-GCM ciphertext)
--   is_encrypted  boolean

-- Entity: AiTool
--   name          machine identifier: 'claude_code' | 'cursor' | 'codex'
--   session_folder absolute path resolved at onboarding

-- Entity: Project
--   tech_stack    detected via marker files (angular.json, .csproj, package.json, pom.xml, requirements.txt, go.mod)
--   has_git       presence of .git folder
--   is_selected   user has opted this project into daily scanning

-- Entity: AiSession
--   session_date  date portion only — used for day-boundary filtering
--   raw_summary   concatenated message text extracted from session file
--   Linked to project via project_id (nullable if unmatched)

-- Entity: GitActivity
--   change_type   classified by the Git Intelligence Module using commit message heuristics
--                 + changed file path patterns
--   insertions / deletions: from `git diff --stat`

-- Entity: Activity
--   Produced by the Activity Engine from AiSession + GitActivity records.
--   confidence_score 0.0–1.0; activities < 0.3 are flagged (is_flagged=1)
--   source_session_id and source_commit_id provide traceability back to raw data

-- Entity: StyleProfile
--   profile_json  see StyleProfile type in docs/research/style-profile-model.md

-- Entity: GeneratedTimesheet
--   content       editable markdown; user may have modified from raw_draft
--   raw_draft     original AI output preserved for diff/comparison
--   activity_ids  JSON array — links back to the Activity records used
--   status        draft → approved → exported
