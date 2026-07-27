# Style Profile Model

## Objective

Define the data model that captures a user's timesheet writing style, learned from
uploaded example timesheets and used as few-shot context for AI generation.

---

## Input Files Supported

| Format | Parser |
|---|---|
| `.txt` | Plain text read directly |
| `.md` | Strip Markdown syntax, read plain text |
| `.docx` | Extract paragraphs via `docx` crate (Rust) or `mammoth` (JS) |
| `.pdf` | Extract text via `pdf-extract` crate (Rust) |

Multiple files can be uploaded. The style profile is computed from all of them combined.

---

## Analysis Areas

### 1. Opening Verbs

Extract the first word of each bullet point. Count frequency.

Example source:
```
- Implemented login screen
- Fixed township dropdown
- Added unit tests for Admin module
- Updated API endpoint for user profile
```

Extracted verbs: `Implemented (1), Fixed (1), Added (1), Updated (1)`

Sorted by frequency → `preferredVerbs: ["Implemented", "Fixed", "Added", "Updated"]`

### 2. Bullet Style

Detect which character starts each list item:
- `-` dash
- `*` asterisk
- `•` bullet
- `1.` numbered

### 3. Grouping Pattern

Detect how tasks are organized:
- `task` — entries grouped under a task/ticket heading
- `project` — entries grouped under project name
- `date` — entries grouped by date
- `flat` — no grouping, plain list

Detection heuristic: look for repeated heading patterns above bullet groups.

Example of task grouping:
```
Task - Admin Screen
- Fixed township dropdown
- Implemented CAMA Neighborhood screen

Task - API Layer
- Added endpoint for profile update
```

### 4. Verbosity Level

Measure average word count per bullet point:
- `low` — 1–5 words
- `medium` — 6–10 words
- `high` — 11+ words

### 5. Sentence Format

Detect whether entries end with punctuation:
- `with_period` — "Fixed the login bug."
- `without_period` — "Fixed the login bug"

### 6. Heading Style

If task-grouped, detect heading format:
- `Task - <name>` (dash separator)
- `<name>:` (colon)
- `[<name>]` (brackets)
- `**<name>**` (bold markdown)

---

## StyleProfile Type

```typescript
interface StyleProfile {
  id:               string;
  userId:           string;

  // Verb preferences (ordered by frequency, most common first)
  preferredVerbs:   string[];

  // List formatting
  bulletChar:       '-' | '*' | '•' | 'numbered';

  // How entries are organized in the output
  groupBy:          'task' | 'project' | 'date' | 'flat';

  // Average detail level per bullet
  verbosity:        'low' | 'medium' | 'high';

  // Whether bullets end with a period
  sentenceFormat:   'with_period' | 'without_period';

  // Heading style when groupBy is 'task' or 'project'
  headingStyle:     'dash' | 'colon' | 'brackets' | 'bold' | 'plain';

  // Raw examples extracted from uploaded files (used as few-shot context)
  examples:         string[];   // 3–5 representative timesheet excerpts

  // Metadata
  sourceFiles:      string[];   // original uploaded filenames
  createdAt:        string;
  updatedAt:        string;
}
```

---

## Default Profile (when no examples uploaded)

```json
{
  "preferredVerbs":  ["Implemented", "Fixed", "Added", "Updated", "Resolved"],
  "bulletChar":      "-",
  "groupBy":         "task",
  "verbosity":       "medium",
  "sentenceFormat":  "without_period",
  "headingStyle":    "dash",
  "examples":        []
}
```

---

## AI Generation Prompt Construction

The style profile is injected into the system prompt of every generation call:

```
You are a timesheet assistant. Generate a daily work update matching this style:

Grouping: by task
Bullet character: -
Verbosity: medium
Preferred opening verbs: Implemented, Fixed, Added
Example timesheets from this user:

---
Task - Admin Screen
- Implemented township dropdown with search
- Fixed CAMA neighborhood validation

Task - API
- Added user profile endpoint
---

Now generate a timesheet for today's work:
<activities>
```

---

## Storage

Stored as JSON in `style_profiles.profile_json`.
One profile per user. Re-computed when new example files are uploaded.
Previous profile is replaced (not versioned) — simplest for MVP.
