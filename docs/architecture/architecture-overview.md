# Architecture Overview

## Application Type

Tauri 2.x desktop application.
Windows 10 / Windows 11.

---

## Layer Diagram

```
┌─────────────────────────────────────────────────────────┐
│                     Angular 20 UI                       │
│  (Components · Signals · Angular Material · Tailwind)   │
├─────────────────────────────────────────────────────────┤
│               Angular Application Core                  │
│  (Feature Services · Repository Clients · State Store)  │
├─────────────────────────────────────────────────────────┤
│                Tauri IPC Command Layer                   │
│   invoke('command_name', payload) → serde_json result   │
├─────────────────────────────────────────────────────────┤
│                    Rust Services                        │
│  (Scanners · Parsers · Git · AI Provider · Scheduler)   │
├──────────────┬──────────────────┬───────────────────────┤
│    SQLite    │   File System    │   External APIs       │
│  (rusqlite)  │  (Tauri plugin)  │  (OpenAI / Az OpenAI) │
└──────────────┴──────────────────┴───────────────────────┘
```

---

## Module Map

### Onboarding Module
- Collects user profile, AI tool selection, project root folders
- Imports previous timesheet examples for style learning
- Runs once; settings persisted to SQLite `settings` table

### Project Discovery Module
- Scans configured root folders
- Detects: Angular, .NET, Node, Java, Python, Go projects
- Detects: Git repositories
- Stores discovered projects in `projects` table

### AI Session Reader Module
- Registry-based: one adapter per AI tool
- Adapters: ClaudeCodeReader, CursorReader, CodexReader
- Each adapter: locates session files → parses JSONL/JSON → emits normalized AiSession records
- Filters to today's date window before storage

### Git Intelligence Module
- Runs `git log`, `git diff`, `git status`, `git branch` via Rust `std::process::Command`
- Extracts: commits, changed files, branch name
- Classifies each change: Feature | BugFix | Refactor | Test | Docs | Chore
- Stores in `git_activities` table

### Activity Engine
- Merges AiSession records + GitActivity records by project + time window
- Deduplicates overlapping entries
- Assigns confidence score (0.0–1.0) per activity
- Outputs normalized `Activity` records

### Style Learning Engine
- Parses uploaded timesheet examples (DOCX/PDF/TXT/MD)
- Extracts: preferred verbs, grouping style, verbosity level, bullet format
- Stores style profile as JSON in `style_profiles` table
- Profile is passed as few-shot context to AI generation step

### Summary / Timesheet Generator
- Calls OpenAI / Azure OpenAI with:
  - System prompt: write in the user's style
  - Context: today's activities, style profile examples
- Returns structured draft: task headings + bullet points
- Stores draft in `generated_timesheets` table with status=draft

### Export Module
- Renders timesheet draft to: plaintext, Markdown, DOCX
- Copy-to-clipboard support
- Does not submit automatically — human approval required

---

## Data Flow

```
[AI Session Files]──────────────┐
                                ▼
[Git Repositories]──────► Activity Engine ──► Style Engine ──► AI Generator
                                                                     │
[Style Profile]─────────────────────────────────────────────────────┘
                                                                     │
                                                                     ▼
                                                          Generated Draft
                                                                     │
                                                          User Review & Edit
                                                                     │
                                                               Export / Copy
```

---

## IPC Pattern

Angular calls Tauri commands via the `@tauri-apps/api/core` invoke function.

```typescript
// Frontend
const projects = await invoke<Project[]>('get_projects');
const sessions  = await invoke<AiSession[]>('scan_sessions', { projectIds });
```

```rust
// Rust backend
#[tauri::command]
async fn get_projects(db: State<'_, DbPool>) -> Result<Vec<Project>, AppError> { ... }
```

All command arguments and return types are serialized as JSON via serde.

---

## State Management

Angular Signals used throughout.
No NgRx. No BehaviorSubject.

Pattern per feature:
```typescript
// Feature store signal
readonly projects = signal<Project[]>([]);
readonly loading  = signal(false);
readonly error    = signal<string | null>(null);
```

---

## Database Strategy

SQLite via `tauri-plugin-sql` (frontend access) for reads.
Writes and sensitive operations go through Rust commands only.
Drizzle ORM used on the TypeScript side for schema definition and query building.
Rust uses `rusqlite` for migration execution and write operations.

Migration files live in `src-tauri/migrations/`.

---

## Security Boundaries

| Action | Layer | Restriction |
|---|---|---|
| Read session files | Rust only | Scoped to user-approved folders |
| Write to SQLite | Rust only | No direct frontend DB writes |
| Store API keys | OS keystore | Never in SQLite plain text |
| Log messages | Rust logger | API keys redacted before logging |
| File system scan | Rust scanner | Blocklist enforced (system dirs excluded) |

---

## Folder Conventions

```
src/app/
├── core/           # App-wide services, guards, interceptors
├── shared/         # Reusable components, pipes, directives
├── layouts/        # Shell, sidebar, topbar components
├── features/       # One folder per major feature
│   ├── onboarding/
│   ├── dashboard/
│   ├── projects/
│   ├── sessions/
│   ├── activities/
│   ├── timesheets/
│   └── settings/
├── models/         # TypeScript interfaces (mirrors Rust structs)
└── store/          # Global signal-based state

src-tauri/src/
├── commands/       # #[tauri::command] handlers (thin, delegate to services)
├── services/       # Business logic
├── repositories/   # SQLite query methods
├── parsers/        # AI session + style file parsers
├── scanners/       # Filesystem + project detection
├── models/         # Rust structs with serde derives
└── utils/          # Error types, logging helpers
```
