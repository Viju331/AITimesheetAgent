# P1-002 - Configure Tauri

## Objective

Convert Angular application into a desktop application using Tauri.

---

# Tasks

Install Tauri:

```bash
npm install @tauri-apps/api
cargo install tauri-cli
```

Initialize:

```bash
cargo tauri init
```

Configure:

* Application Name
* Application Identifier
* Window Size

---

## Verify

Run:

```bash
npm run tauri dev
```

Acceptance:

* Desktop window launches
* Angular UI renders
* No Rust errors

---

# Deliverables

* Tauri Integration
* Desktop Window
* Development Workflow

---

# Acceptance Criteria

* Angular runs inside Tauri
* Hot reload works
* Build succeeds
