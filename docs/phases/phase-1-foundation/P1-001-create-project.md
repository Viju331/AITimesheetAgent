# P1-001 - Create Project

## Objective

Create the base Angular + Tauri application.

---

# Tasks

## Install Prerequisites

Install:

* Node.js LTS
* Angular CLI
* Rust
* Visual Studio Build Tools
* Git

Verify:

```bash
node -v
npm -v
rustc -V
cargo -V
git --version
```

---

## Create Angular Application

```bash
ng new ai-timesheet-assistant
```

Configuration:

* Standalone Components = Yes
* Routing = Yes
* CSS = SCSS

---

## Verify Angular Application

Run:

```bash
ng serve
```

Acceptance:

* Application loads
* No console errors
* No build warnings

---

# Deliverables

* Angular Workspace
* Git Repository
* Initial Commit

---

# Acceptance Criteria

* Application starts successfully
* Routing enabled
* Standalone architecture enabled
