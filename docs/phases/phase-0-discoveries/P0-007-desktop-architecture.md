# P0-007 - Desktop Architecture

## Objective

Define the high-level architecture of the AI Timesheet Assistant desktop application.

---

# Architecture Overview

```text
┌───────────────────────────────┐
│         Angular UI            │
├───────────────────────────────┤
│        Application Core       │
├───────────────────────────────┤
│      Tauri Command Layer      │
├───────────────────────────────┤
│         Rust Services         │
├───────────────────────────────┤
│ SQLite │ File System │ Git    │
└───────────────────────────────┘
```

---

# Core Modules

## Onboarding Module

Responsibilities:

* First-time setup
* AI tool configuration
* Project folder configuration
* Timesheet import

---

## Project Discovery Module

Responsibilities:

* Detect projects
* Detect repositories
* Detect AI tool workspaces

---

## AI Session Reader Module

Responsibilities:

* Read Claude sessions
* Read Cursor sessions
* Read Codex sessions
* Normalize data

---

## Git Intelligence Module

Responsibilities:

* Analyze commits
* Analyze modified files
* Build activity timeline

---

## Activity Engine

Responsibilities:

* Merge all activities
* Create daily timeline
* Remove duplicates

---

## Style Learning Engine

Responsibilities:

* Learn writing patterns
* Learn formatting
* Learn grouping style

---

## Summary Engine

Responsibilities:

* Generate work summaries
* Generate timesheets
* Generate exports

---

# Data Flow

```text
AI Sessions
        │
Git Activity
        │
        ▼
Activity Engine
        │
        ▼
Summary Engine
        │
        ▼
Timesheet Generator
        │
        ▼
Export
```

---

# Design Principles

* Offline First
* Local Data Ownership
* Modular Architecture
* Extensible AI Connectors
* Provider Independent
