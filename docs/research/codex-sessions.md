# Codex (OpenAI Codex CLI) — Session Format Research

## Status

Primary MVP integration. Format requires on-machine verification before parser implementation.

---

## Product Note

"Codex" in this context refers to the **OpenAI Codex CLI** (open source, released 2025),
not the deprecated OpenAI Codex API model.
GitHub: https://github.com/openai/codex

---

## Storage Location

```
Windows:   %USERPROFILE%\.codex\
macOS:     ~/.codex/
Linux:     ~/.codex/
```

### Folder Structure (expected)

```
~/.codex/
├── config.toml       # CLI configuration (model, provider)
└── history/
    └── <session-id>.json   # One file per conversation session
```

The exact sub-folder name may vary. If `history/` does not exist, check:
- `~/.codex/sessions/`
- `~/.codex/logs/`

---

## Session File Format (expected)

Each session file is a JSON file containing the full conversation.

```json
{
  "id": "session-uuid",
  "model": "o4-mini",
  "created_at": "2025-06-15T09:00:00Z",
  "cwd": "/path/to/project",
  "messages": [
    {
      "role": "user",
      "content": "implement the login screen",
      "timestamp": "2025-06-15T09:00:05Z"
    },
    {
      "role": "assistant",
      "content": "I'll implement the login screen...",
      "timestamp": "2025-06-15T09:00:10Z"
    }
  ]
}
```

**Note:** The exact schema is unconfirmed. The above is the expected format based on
common CLI tool patterns. Must be verified by inspecting actual `~/.codex/` output.

---

## Parsing Strategy

### Step 1 — Locate session directory

Check for `~/.codex/history/`, `~/.codex/sessions/`, and `~/.codex/` root.
Use whichever contains `.json` files.

### Step 2 — Date filtering

Filter session files by `created_at` date OR by individual `messages[].timestamp` date.
Only process sessions where at least one message falls on the target date.

### Step 3 — Project mapping

If the session JSON contains a `cwd` field, match against known project paths.
If no `cwd` field: attempt to match based on file/folder names mentioned in message text.

### Step 4 — Extract user messages

Only `role == "user"` entries contain developer intent.
Concatenate for `raw_summary`.

### Step 5 — Normalize

Emit one `AiSession` record per (project, date, session file).

---

## Fallback: No Local Storage

If Codex CLI does not persist sessions locally (determined at verification time),
the adapter returns empty and logs a warning. No error is thrown — the session
source is simply unavailable.

---

## Verification Steps Required

1. Install Codex CLI on developer machine
2. Run a session and inspect `~/.codex/` for created files
3. Document actual folder name, file format, timestamp field name
4. Confirm whether `cwd` is included in session metadata
5. Update this document with confirmed findings before implementing the parser

---

## Implementation Note

The `CodexReader` adapter must implement the same `AiToolReader` trait as
`ClaudeCodeReader` and `CursorReader`. If format differs significantly from
the expected schema above, only the parser internals change — the adapter
interface stays the same.
