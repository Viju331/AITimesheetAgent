# P0-009 - Folder Structure

## Objective

Define the project structure.

---

# Frontend

```text
src/
│
├── app/
│   ├── core/
│   ├── shared/
│   ├── layouts/
│   ├── pages/
│   ├── features/
│   ├── services/
│   ├── models/
│   ├── guards/
│   └── store/
│
├── assets/
├── environments/
└── styles/
```

---

# Feature Modules

```text
features/
│
├── onboarding/
├── dashboard/
├── projects/
├── sessions/
├── activities/
├── summaries/
├── timesheets/
├── settings/
└── analytics/
```

---

# Tauri

```text
src-tauri/
│
├── src/
│   ├── commands/
│   ├── services/
│   ├── repositories/
│   ├── parsers/
│   ├── scanners/
│   ├── models/
│   └── utils/
│
├── capabilities/
└── migrations/
```

---

# Documentation

```text
docs/
│
├── phases/
├── architecture/
├── database/
├── api/
└── research/
```
