# P0-014 - API Contract Design

## Objective

Design internal Angular ↔ Tauri communication.

---

# Command: Get Projects

```typescript
getProjects()
```

Response:

```json
[
  {
    "id": "",
    "name": "",
    "path": ""
  }
]
```

---

# Command: Scan Sessions

```typescript
scanSessions()
```

---

# Command: Analyze Git

```typescript
analyzeGit()
```

---

# Command: Generate Summary

```typescript
generateSummary()
```

---

# Command: Generate Timesheet

```typescript
generateTimesheet()
```

---

# Command: Export Timesheet

```typescript
exportTimesheet()
```

---

# Future API

Potential cloud sync support.

Not included in MVP.
