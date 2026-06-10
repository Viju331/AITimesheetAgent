# P1-012 - SQLite Setup

## Objective

Configure SQLite database for local storage.

---

# Tasks

Research and select:

Option A:

```text
tauri-plugin-sql
```

Option B:

```text
rusqlite
```

Preferred:

```text
rusqlite
```

---

# Create Database

Location:

```text
%APPDATA%/AI-Timesheet/database.db
```

---

# Create Migration System

Folder:

```text
src-tauri/migrations
```

---

# Migration Naming

```text
001_initial_schema.sql

002_add_projects.sql

003_add_sessions.sql
```

---

# Deliverables

* SQLite Database
* Migration Runner

---

# Acceptance Criteria

Application creates database automatically on startup.
