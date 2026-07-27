# Data Flow

## Onboarding Flow

```
User Launch (first run)
        │
        ▼
Onboarding Wizard
  ├── Enter name / email
  ├── Select AI tools (Claude / Cursor / Codex)
  ├── Pick project root folders
  └── Upload previous timesheets (optional)
        │
        ▼
Settings saved to SQLite
Style profile computed and saved
        │
        ▼
Dashboard (ready)
```

---

## Daily Generate Flow

```
User opens app
        │
        ▼
Dashboard — select date (default: today)
        │
        ▼
[Scan AI Sessions]────────────────────────────────────────┐
  For each enabled AI tool:                               │
    Locate session files in configured folder             │
    Filter entries with timestamp = today                 │
    Parse messages → extract tasks/actions                │
    Normalize to AiSession[]                              │
                                                          │
[Scan Git Activity]───────────────────────────────────────┤
  For each selected project with a git repo:              │
    git log --since=midnight --author=current-user        │
    git diff HEAD~n (for each commit)                     │
    Classify each commit (Feature/BugFix/Refactor/Test)   │
    Normalize to GitActivity[]                            │
                                                          ▼
                                              Activity Engine
                                                Merge by project
                                                Deduplicate
                                                Score confidence
                                                Emit Activity[]
                                                          │
                                                          ▼
                                              Style Engine
                                                Load style profile
                                                Build few-shot examples
                                                          │
                                                          ▼
                                              AI Generator
                                                POST /chat/completions
                                                system: style instructions
                                                user: today's activities
                                                          │
                                                          ▼
                                              Draft saved (status=draft)
                                                          │
                                                          ▼
                                              User reviews draft in UI
                                              User edits inline
                                                          │
                                                          ▼
                                              Export (copy / DOCX / MD)
```

---

## Style Learning Flow

```
User uploads timesheet examples
        │
        ▼
File parser (DOCX / PDF / TXT / MD)
        │
        ▼
Extract text content
        │
        ▼
Analyze:
  - Common opening verbs (Implemented, Fixed, Added, Updated...)
  - Sentence length distribution
  - Grouping pattern (by task / by project / by date)
  - Bullet style (dash / asterisk / number)
  - Verbosity level (short / medium / detailed)
        │
        ▼
StyleProfile JSON saved to style_profiles table
        │
        ▼
Used as few-shot system context in every generation call
```

---

## Confidence Score Rules

| Signal | Score Contribution |
|---|---|
| Activity confirmed by both AI session AND git commit | +0.4 |
| Activity from AI session only | +0.2 |
| Activity from git commit only | +0.2 |
| Commit message clearly describes the work | +0.2 |
| AI session message explicitly names a task/ticket | +0.2 |
| File change matches known feature folder | +0.1 |
| Timestamp is within core working hours (8am–7pm) | +0.05 |

Score range: 0.0 – 1.0
Activities below 0.3 are flagged for user review before generation.
