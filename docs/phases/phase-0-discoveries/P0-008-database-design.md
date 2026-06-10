# P0-008 - Database Design

## Objective

Define the SQLite database structure for the MVP.

---

# Table: users

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT,
    created_at DATETIME
);
```

---

# Table: settings

```sql
CREATE TABLE settings (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL,
    value TEXT NOT NULL
);
```

---

# Table: projects

```sql
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    source_type TEXT NOT NULL,
    created_at DATETIME
);
```

---

# Table: ai_sessions

```sql
CREATE TABLE ai_sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT,
    source TEXT,
    session_date DATETIME,
    summary TEXT,
    file_path TEXT
);
```

---

# Table: git_activities

```sql
CREATE TABLE git_activities (
    id TEXT PRIMARY KEY,
    project_id TEXT,
    commit_hash TEXT,
    commit_date DATETIME,
    commit_message TEXT
);
```

---

# Table: activities

```sql
CREATE TABLE activities (
    id TEXT PRIMARY KEY,
    project_id TEXT,
    activity_type TEXT,
    title TEXT,
    description TEXT,
    created_at DATETIME
);
```

---

# Table: style_profiles

```sql
CREATE TABLE style_profiles (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    profile_json TEXT,
    created_at DATETIME
);
```

---

# Table: generated_timesheets

```sql
CREATE TABLE generated_timesheets (
    id TEXT PRIMARY KEY,
    generated_date DATETIME,
    content TEXT,
    created_at DATETIME
);
```

---

# Relationships

```text
Users
   │
   ▼
Style Profiles

Projects
   │
   ├── AI Sessions
   │
   ├── Git Activities
   │
   └── Activities
```
