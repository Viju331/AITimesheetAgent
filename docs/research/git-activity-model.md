# Git Activity Model

## Objective

Define how raw git output is transformed into structured, classified activity records.

---

## Commands Used

```bash
# All commits by current user since midnight local time
git log --since="midnight" --author="<user.email>" \
        --format="%H|%ae|%an|%aI|%s" --no-merges

# Files changed in a specific commit
git diff-tree --no-commit-id -r --name-status <commit-hash>

# Stat summary (insertions/deletions)
git show --stat --format="" <commit-hash>

# Current branch
git rev-parse --abbrev-ref HEAD

# Working tree status (unstaged / staged changes)
git status --porcelain
```

All commands are invoked from Rust via `std::process::Command` with the project path
as the working directory. Never shell-expand user input into command arguments.

---

## Raw Data Model

### Commit Entry (from git log)

```
HASH      | ae | an    | aI (ISO 8601)          | subject
abc123def | u@e.com | User Name | 2025-06-15T09:32:00+10:00 | fix township dropdown
```

### File Change Entry (from git diff-tree)

```
STATUS  FILEPATH
M       src/app/features/admin/township.component.ts
A       src/app/features/admin/township.component.spec.ts
D       src/app/features/admin/old-dropdown.component.ts
```

Status codes: `A` = Added, `M` = Modified, `D` = Deleted, `R` = Renamed, `C` = Copied

---

## Change Type Classification

### Rules (applied in order, first match wins)

| Rule | Detected Type |
|---|---|
| Commit message starts with `fix`, `bug`, `hotfix`, `resolve`, `patch` (case-insensitive) | `bugfix` |
| Commit message starts with `feat`, `add`, `implement`, `create`, `new` | `feature` |
| Commit message starts with `refactor`, `rename`, `move`, `restructure`, `cleanup` | `refactor` |
| Commit message starts with `test`, `spec`, `coverage` | `test` |
| Commit message starts with `docs`, `doc`, `readme`, `changelog` | `docs` |
| Commit message starts with `chore`, `build`, `ci`, `deps`, `bump` | `chore` |
| Any `.spec.ts` / `_test.py` / `Test.java` file in changed files | `test` |
| Any `README` / `.md` / `docs/` in changed files | `docs` |
| Default | `feature` |

### File Path Patterns (supplement commit message rules)

| File Pattern | Reinforces Type |
|---|---|
| `*.spec.ts`, `*_test.*`, `*Test.java`, `*Tests.cs` | `test` |
| `*.md`, `docs/`, `README` | `docs` |
| `*.config.*`, `*.json`, `Dockerfile`, `*.yml` | `chore` |
| Files in `bugfix/` or `hotfix/` branch name | `bugfix` |
| Files in `feature/` branch name | `feature` |

---

## GitActivity Record

```typescript
interface GitActivity {
  id:             string;         // UUID v4
  projectId:      string;
  commitHash:     string;
  branch:         string;
  commitDate:     string;         // ISO 8601 UTC
  commitMessage:  string;         // subject line only
  authorName:     string;
  authorEmail:    string;
  filesChanged:   number;
  insertions:     number;
  deletions:      number;
  changeType:     ChangeType;
  changedFiles:   ChangedFile[];  // populated in memory, not stored flat in DB
}

type ChangeType = 'feature' | 'bugfix' | 'refactor' | 'test' | 'docs' | 'chore';

interface ChangedFile {
  status:    'added' | 'modified' | 'deleted' | 'renamed' | 'copied';
  path:      string;
  extension: string;
  folder:    string;
}
```

---

## Activity Title Generation

After classification, generate a human-readable title from the commit:

| Change Type | Title Template |
|---|---|
| `feature` | `Implemented <commit subject>` |
| `bugfix` | `Fixed <commit subject>` |
| `refactor` | `Refactored <commit subject>` |
| `test` | `Added tests for <commit subject>` |
| `docs` | `Updated documentation: <commit subject>` |
| `chore` | `<commit subject>` (unchanged) |

Strip conventional commit prefixes (`feat:`, `fix:`, `test:`, etc.) before applying template.

---

## Confidence Score Contribution

| Signal | Score |
|---|---|
| Commit message is longer than 10 characters | +0.15 |
| Commit message references a ticket (JIRA-123, ADO-456, #123) | +0.20 |
| Change type was determined from commit message (not file patterns only) | +0.10 |
| Branch name is a feature/bugfix branch (not `main`/`develop`) | +0.10 |
| Same work also appears in AI session for the same project+date | +0.40 |

---

## Author Filtering

Only process commits where `author_email` matches the user's email (from `users.email`).
If `users.email` is not set, fall back to matching `git config user.email` at runtime.
Include commits from `users.name` if email is unavailable.

---

## Date Boundary

"Today" = local date from midnight 00:00:00 to 23:59:59 in the user's local timezone.
Use `--since="midnight"` in git log, which respects local timezone automatically.

For scanning a specific past date:
```bash
git log --after="2025-06-14T00:00:00" --before="2025-06-15T00:00:00" ...
```
