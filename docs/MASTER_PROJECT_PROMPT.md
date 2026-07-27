# AI Timesheet Intelligence Platform - Master Development Prompt

## Role

You are acting as a Principal Software Architect, Senior Full Stack Developer, AI Systems Engineer, and Technical Project Lead.

Your responsibility is to help design and build a production-ready desktop application from scratch.

You must think like an architect first, then implement like a senior engineer.

Do not jump directly into coding.
First understand architecture, dependencies, risks, and implementation strategy.

---

# Project Name

AI Timesheet Intelligence Platform

# Product Vision

Build a desktop application that automatically understands a developer's daily work and generates professional timesheet updates.

The application collects information from:

- AI coding assistants
- Local development projects
- Git repositories
- Task management systems
- Communication tools
- Calendar
- Work tracking systems

The system converts raw developer activity into structured, human-readable timesheet updates.

---

# Core Problem

Developers spend time manually writing daily updates.

The application should automatically understand:

- What the developer worked on
- Which project they worked on
- Which tasks they completed
- What code changes happened
- What discussions happened
- What tickets are assigned
- What meetings happened

Then generate a timesheet update matching the user's writing style.

---

# Target Users

Software developers.

Each user may use different AI tools:

- Claude Code
- Cursor
- Codex
- GitHub Copilot
- Cline
- Continue
- Other future AI agents

---

# Application Type

Desktop Application.

Reason:

The application needs access to local developer data:

Examples:

AI tool folders:

- ~/.claude
- ~/.cursor
- ~/.codex
- ~/.copilot

Project folders:

- Source code repositories
- Git folders
- Workspace files

The application must safely read local files with user permission.

---

# Technology Stack

## Desktop

Use:

Tauri 2.x

Reason:

- Lightweight
- Secure
- Native filesystem access
- Better performance than Electron
- Rust backend

## Frontend

Use:

Angular latest stable version

Architecture:

Standalone components

UI:

Angular Material

Tailwind CSS

State Management:

Angular Signals

Forms:

Reactive Forms

Testing:

Vitest

E2E:

Playwright

---

## Backend Layer

Tauri Rust backend.

Responsibilities:

- Filesystem access
- Folder scanning
- Git operations
- Database operations
- Secure commands

Language:

Rust

---

## Database

Local database:

SQLite

Access layer:

Repository Pattern

Migration based schema.

---

# Architecture Principles

Follow:

- Clean Architecture
- SOLID
- Separation of concerns
- Repository pattern
- Service layer
- Dependency injection
- Feature based modules

---

# High Level Architecture

Angular UI

        |

Application Services

        |

Tauri Commands

        |

Rust Services

        |

Repositories

        |

SQLite

---

# Main Modules

## 1. Onboarding Module

Responsibilities:

First time setup.

Collect:

- User details
- AI tools used
- AI tool folders
- Project folders
- Task systems used

---

## 2. Project Discovery Module

Responsibilities:

Scan developer folders.

Detect projects:

Angular

.NET

Node

Java

Python

Git repositories

Store:

Project name

Path

Technology

Repository information

---

## 3. AI Session Intelligence Module

Read AI assistant sessions.

Supported:

Claude

Cursor

Codex

Process:

Find session files

Read timestamps

Filter today's activity

Extract messages

Extract actions

Important:

Only process selected projects.

Never scan unrelated folders.

---

## 4. Git Intelligence Module

Analyze:

Repositories

Branches

Commits

Changed files

Diffs

Detect:

Feature work

Bug fixes

Refactoring

Testing

---

## 5. Activity Intelligence Engine

Combine:

AI sessions

Git changes

Tasks

Calendar

Messages

Create normalized activities.

Example:

Input:

AI:
"Fix township dropdown"

Git:

Modified:
township.component.ts

Output:

Activity:

Fixed Township dropdown issue.

---

## 6. Task Correlation Engine

Connect activities with:

Jira

Azure DevOps

Planner

Other systems

Example:

Ticket:

ADMIN-123

Activities:

Implemented UI

Added API

Added Tests

---

## 7. Writing Style Learning

The user provides previous timesheet examples.

The system learns:

Writing style

Bullet style

Verb usage

Length

Formatting

Example:

User style:

"Implemented..."
"Fixed..."
"Added..."

Generated output should follow the same style.

---

## 8. Timesheet Generator

Generate:

Task Heading

Description bullets

Example:

Task - Admin Screen

- Fixed Township dropdown issue
- Implemented CAMA Neighborhood screen
- Added unit tests

---

## 9. Integrations

Support:

Jira

Azure DevOps

Microsoft Planner

Slack

Microsoft Teams

Outlook

Calendar

Use:

OAuth

Microsoft Graph where applicable

---

## 10. Submission Automation

Support:

Company timesheet portals.

Examples:

SharePoint based timesheets

Internal applications

Capabilities:

Generate

Review

Edit

Submit

---

# Security Requirements

Never:

Upload local files without permission.

Never:

Read outside selected folders.

Credentials:

Store securely.

Encrypt tokens.

---

# Data Privacy

Local-first approach.

Default:

Data stays on device.

Cloud sync optional only.

---

# Development Standards

Always:

Write clean production code.

Add comments only where useful.

Avoid unnecessary complexity.

Every feature requires:

- Implementation
- Unit tests
- Error handling
- Logging
- Documentation

---

# Coding Standards

Angular:

Use:

- Standalone components
- Signals
- Typed models
- Feature folders

Avoid:

- Any type
- Large components
- Business logic inside components

---

Rust:

Use:

- Proper Result handling
- Error enums
- Modular services

Avoid:

- unwrap()
- unsafe code

---

# Folder Structure

Frontend:

src/app

core

shared

features

services

models

store

Backend:

src-tauri/src

commands

services

repositories

models

utils

---

# Development Workflow

For every phase:

1. Analyze requirement

2. Create implementation plan

3. Identify files

4. Implement

5. Add tests

6. Validate

7. Document

---

# Phase Execution Rules

When given a phase task file:

First output:

## Understanding

Explain what needs to be built.

Then:

## Implementation Plan

List steps.

Then:

## Code Changes

Implement.

Then:

## Testing

Add tests.

Then:

## Verification Checklist

---

# Current Project Roadmap

Phase 0:

Project definition and architecture

Phase 1:

Foundation

Tauri

Angular

SQLite

Logging

Testing

Phase 2:

Onboarding

AI tool selection

Project discovery

Phase 3:

AI session parsing

Phase 4:

Git intelligence

Phase 5:

Timesheet generation

Phase 6:

External integrations

Phase 7:

Submission automation

Phase 8:

AI Work Intelligence

---

# Important Product Rules

1. The application must be desktop-first.

2. User controls which folders are scanned.

3. Never assume AI tool storage formats.

4. Build adapters/readers for every AI tool.

5. Keep parsers replaceable.

6. Every activity must have a confidence score.

7. Generated timesheets must always be editable.

8. Human approval required before submission.

---

# Your First Task

Before implementing anything:

Analyze this entire architecture.

Provide:

1. Recommended final architecture diagram

2. Folder structure

3. Database design

4. Core entities

5. Risks and solutions

6. Development order

7. MVP recommendation

Do not write code until architecture review is complete.
