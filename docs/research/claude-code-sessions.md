# Claude Code — Session Format Research

## Status

Primary MVP integration. Confirmed format.

---

## Storage Location

```
Windows:   %USERPROFILE%\.claude\
macOS:     ~/.claude/
Linux:     ~/.claude/
```

### Folder Structure

```
~/.claude/
├── settings.json           # Claude Code app settings
├── settings.local.json     # Local overrides (gitignored by Claude)
└── projects/
    └── <encoded-project-path>/
        ├── <session-uuid-1>.jsonl
        ├── <session-uuid-2>.jsonl
        └── ...
```

The `<encoded-project-path>` directory name is derived from the absolute path of the
project opened in Claude Code. The encoding replaces path separators with `-` and
strips leading slashes.

Example:
- Project path: `/home/user/code/my-app`
- Encoded folder: `-home-user-code-my-app`

---

## Session File Format

Each `.jsonl` file contains one JSON object per line.
Each line is one conversation entry (user message or assistant response).

### Entry Schema

```jsonl
{
  "uuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "parentUuid": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" | null,
  "sessionId": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "type": "user" | "assistant",
  "timestamp": "2025-06-15T09:32:14.123Z",
  "cwd": "/absolute/path/to/project",
  "version": "1.x.x",
  "isSidechain": false,
  "userType": "human" | "tool_result",
  "message": {
    "role": "user" | "assistant",
    "content": "string" | [{ "type": "text", "text": "string" }]
  }
}
```

### Key Fields for Parsing

| Field | Purpose |
|---|---|
| `timestamp` | ISO 8601 UTC — used for date filtering |
| `cwd` | Maps the session to a project by matching to known project paths |
| `type` | `"user"` entries contain developer prompts (task descriptions) |
| `message.content` | The actual text to extract for activity summarization |
| `sessionId` | Groups all entries that belong to one conversation |
| `uuid` | Unique ID per entry |

---

## Parsing Strategy

### Step 1 — Locate session folder

```
CLAUDE_DIR = $HOME/.claude/projects/
```

Map each sub-folder name back to a project path by reversing the encoding:
- Replace leading `-` with `/`
- Replace remaining `-` with the OS path separator where a known project path matches

Or: compare `cwd` field in first entry of each JSONL file against known project paths.

**Recommended approach:** Use `cwd` field — more reliable than folder name decoding.

### Step 2 — Date filtering

Filter lines where `timestamp` date portion equals the target date.

```rust
let target_date = chrono::Local::now().date_naive();
let entry_date = entry.timestamp.date_naive();
if entry_date != target_date { continue; }
```

### Step 3 — Extract user messages

Only `type == "user"` entries contain developer intent.
`type == "assistant"` entries can optionally be included for context summarization.

### Step 4 — Normalize

Emit one `AiSession` record per (project, date, session_file).
Set `raw_summary` to concatenated user message text for that session.

---

## Project Mapping

Match `cwd` field to the `path` column in the `projects` table.
Exact match preferred. If no exact match: check if `cwd` starts with a known project path (subdirectory case).

---

## Limitations

- Session files are never deleted by Claude Code — historical data accumulates
- Very large sessions (100+ turns) may take time to parse; apply a line limit if needed
- `cwd` may be empty for sessions started outside a project context — skip these
- Format may change across Claude Code versions; treat the parser as versioned

---

## Verification Needed

- Confirm exact folder name encoding for Windows paths (`C:\Users\...`)
- Confirm behavior when project is opened via workspace vs single folder
- Test with Claude Code versions prior to 1.x
