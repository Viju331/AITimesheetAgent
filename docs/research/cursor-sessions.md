# Cursor — Session Format Research

## Status

Primary MVP integration. Format partially confirmed — verification required.

---

## Storage Location

Cursor is built on VS Code. Session/chat data is stored in the VS Code extension
host storage area, per workspace.

```
Windows:   %APPDATA%\Cursor\User\workspaceStorage\<workspace-hash>\
macOS:     ~/Library/Application Support/Cursor/User/workspaceStorage/<workspace-hash>/
Linux:     ~/.config/Cursor/User/workspaceStorage/<workspace-hash>/
```

Global user data:
```
Windows:   %APPDATA%\Cursor\User\globalStorage\
```

---

## Workspace Hash

The `<workspace-hash>` directory name is an MD5 hash of the workspace URI.

Example:
- Workspace path: `C:\Users\user\code\my-app`
- URI: `file:///c%3A/Users/user/code/my-app`
- Hash: MD5 of the URI string (lowercase hex)

To find the correct workspace folder, enumerate all `workspaceStorage/<hash>/`
directories and check for a `workspace.json` file inside each — it contains the
original workspace path.

---

## Chat Storage File

Within the workspace storage folder, Cursor stores chat history in:

```
workspaceStorage/<hash>/state.vscdb       # SQLite database (VS Code state)
```

The `state.vscdb` is a SQLite file. Cursor stores chat panels as serialized JSON
blobs inside the VS Code `ItemTable`.

### Query to Extract Chat Data

```sql
SELECT key, value
FROM ItemTable
WHERE key LIKE '%cursor%chat%'
   OR key LIKE '%aichat%'
   OR key LIKE '%composer%';
```

The exact key varies by Cursor version. Common keys observed:
- `aiService.chatHistory`
- `cursor.chatPanelState`
- `workbench.panel.aichat.view.aichat.chatdata`

### Chat Entry Schema (approximate)

```json
{
  "tabs": [
    {
      "tabId": "uuid",
      "chatTitle": "optional title",
      "lastSendTime": 1718437934000,
      "bubbles": [
        {
          "type": "user" | "ai",
          "text": "message text",
          "timestamp": 1718437934000
        }
      ]
    }
  ]
}
```

---

## Parsing Strategy

### Step 1 — Locate workspace folders

Enumerate `%APPDATA%\Cursor\User\workspaceStorage\` sub-directories.
For each, read `workspace.json` to resolve the original workspace path.
Match against known project paths in `projects` table.

### Step 2 — Open state.vscdb

Use a SQLite reader (from Rust via `rusqlite`) to query the `ItemTable`.
Search for chat-related keys (see query above).

### Step 3 — Parse JSON blob

Deserialize the JSON value from the matching row.
Iterate `tabs[].bubbles[]` where `type == "user"`.
Filter by `timestamp` >= today midnight epoch.

### Step 4 — Normalize

Emit one `AiSession` record per (project, date).
Set `raw_summary` to concatenated user bubble text for that day.

---

## Alternative: Global Chat Storage

Cursor also maintains global chat history independent of workspace:
```
%APPDATA%\Cursor\User\globalStorage\cursor.storage\chat\
```

Files here may be JSON files named by session ID. Check this location if
workspace-scoped storage yields no results.

---

## Limitations

- `state.vscdb` is a live SQLite file; use WAL mode and read-only connection to avoid locking
- Schema keys change across Cursor versions — build a resilient key scanner
- `timestamp` in bubbles is Unix epoch milliseconds, not ISO 8601
- Some Cursor versions store conversations only in memory (not persisted) until export

---

## Verification Needed

- Confirm exact JSON key names for Cursor version in use
- Confirm chat persistence behavior (some versions require manual export)
- Test workspace path resolution on Windows with spaces in path
- Check if Cursor stores history in `cursor.chat` extension storage vs VS Code host storage
