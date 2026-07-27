/**
 * Core domain models for the AI Timesheet Intelligence Platform.
 *
 * These interfaces mirror the Rust structs in src-tauri/src/models/
 * and the SQLite schema in docs/database/migrations/001-initial-schema.sql.
 *
 * All dates are ISO 8601 strings. All IDs are UUID v4 strings.
 */

// ============================================================
// Enumerations
// ============================================================

export type ChangeType =
  | 'feature'
  | 'bugfix'
  | 'refactor'
  | 'test'
  | 'docs'
  | 'chore';

export type ActivityType = ChangeType | 'meeting';

export type TechStack =
  | 'angular'
  | 'dotnet'
  | 'node'
  | 'java'
  | 'python'
  | 'go'
  | 'unknown';

export type AiToolName = 'claude_code' | 'cursor' | 'codex';

export type TimesheetStatus = 'draft' | 'approved' | 'exported';

export type AiProvider = 'openai' | 'azure_openai';

export type GroupBy = 'task' | 'project' | 'date' | 'flat';

export type Verbosity = 'low' | 'medium' | 'high';

export type BulletChar = '-' | '*' | '•' | 'numbered';

export type HeadingStyle = 'dash' | 'colon' | 'brackets' | 'bold' | 'plain';

export type SentenceFormat = 'with_period' | 'without_period';

export type FileStatus = 'added' | 'modified' | 'deleted' | 'renamed' | 'copied';

// ============================================================
// User
// ============================================================

export interface User {
  id:        string;
  name:      string;
  email:     string | null;
  createdAt: string;
}

export interface CreateUserRequest {
  name:  string;
  email: string | null;
}

// ============================================================
// Setting
// ============================================================

export interface Setting {
  id:          string;
  key:         string;
  value:       string;
  isEncrypted: boolean;
  updatedAt:   string;
}

export interface UpsertSettingRequest {
  key:         string;
  value:       string;
  isEncrypted: boolean;
}

// ============================================================
// AI Tool
// ============================================================

export interface AiTool {
  id:            string;
  name:          AiToolName;
  displayName:   string;
  sessionFolder: string | null;
  isEnabled:     boolean;
  createdAt:     string;
}

export interface UpdateAiToolRequest {
  id:            string;
  sessionFolder: string | null;
  isEnabled:     boolean;
}

// ============================================================
// Project
// ============================================================

export interface Project {
  id:          string;
  name:        string;
  path:        string;
  techStack:   TechStack;
  hasGit:      boolean;
  gitRemote:   string | null;
  isSelected:  boolean;
  createdAt:   string;
  lastScanned: string | null;
}

export interface UpdateProjectSelectionRequest {
  id:         string;
  isSelected: boolean;
}

// ============================================================
// AI Session
// ============================================================

export interface AiSession {
  id:          string;
  projectId:   string | null;
  toolId:      string | null;
  sessionDate: string;           // ISO date: '2025-06-15'
  filePath:    string;
  rawSummary:  string | null;
  parsedAt:    string;
}

// ============================================================
// Git Activity
// ============================================================

export interface ChangedFile {
  status:    FileStatus;
  path:      string;
  extension: string;
  folder:    string;
}

export interface GitActivity {
  id:            string;
  projectId:     string;
  commitHash:    string;
  branch:        string | null;
  commitDate:    string;
  commitMessage: string;
  authorName:    string | null;
  authorEmail:   string | null;
  filesChanged:  number;
  insertions:    number;
  deletions:     number;
  changeType:    ChangeType;
  changedFiles:  ChangedFile[];  // populated in memory on read
  scannedAt:     string;
}

// ============================================================
// Activity (normalized, deduplicated)
// ============================================================

export interface Activity {
  id:              string;
  projectId:       string | null;
  activityDate:    string;           // ISO date
  activityType:    ActivityType;
  title:           string;
  description:     string | null;
  sourceSessionId: string | null;
  sourceCommitId:  string | null;
  confidenceScore: number;           // 0.0–1.0
  isFlagged:       boolean;
  createdAt:       string;
}

// ============================================================
// Style Profile
// ============================================================

export interface StyleProfile {
  id:             string;
  userId:         string;
  preferredVerbs: string[];
  bulletChar:     BulletChar;
  groupBy:        GroupBy;
  verbosity:      Verbosity;
  sentenceFormat: SentenceFormat;
  headingStyle:   HeadingStyle;
  examples:       string[];     // 3–5 representative excerpts used as few-shot context
  sourceFiles:    string[];
  createdAt:      string;
  updatedAt:      string;
}

// ============================================================
// Generated Timesheet
// ============================================================

export interface GeneratedTimesheet {
  id:            string;
  userId:        string | null;
  generatedDate: string;          // ISO date the timesheet covers
  content:       string;          // Markdown — editable by user
  rawDraft:      string | null;   // original AI output
  status:        TimesheetStatus;
  activityIds:   string[];        // source activity IDs
  aiProvider:    AiProvider | null;
  aiModel:       string | null;
  createdAt:     string;
  exportedAt:    string | null;
}

export interface UpdateTimesheetContentRequest {
  id:      string;
  content: string;
  status:  TimesheetStatus;
}

// ============================================================
// Scan / Generation Requests & Responses
// ============================================================

export interface ScanSessionsRequest {
  projectIds: string[];
  date:       string;   // ISO date: '2025-06-15'
}

export interface ScanSessionsResponse {
  sessions:    AiSession[];
  sessionCount: number;
  errors:      string[];
}

export interface AnalyzeGitRequest {
  projectIds: string[];
  date:       string;
}

export interface AnalyzeGitResponse {
  activities:    GitActivity[];
  activityCount: number;
  errors:        string[];
}

export interface GenerateTimesheetRequest {
  date:           string;
  projectIds:     string[];
  activityIds:    string[];
  styleProfileId: string | null;
}

export interface GenerateTimesheetResponse {
  timesheet: GeneratedTimesheet;
  tokenUsage: {
    promptTokens:     number;
    completionTokens: number;
    totalTokens:      number;
  } | null;
}

export interface ExportTimesheetRequest {
  timesheetId: string;
  format:      'markdown' | 'plaintext' | 'docx';
  outputPath:  string | null;   // null = copy to clipboard
}

export interface ExportTimesheetResponse {
  success:    boolean;
  outputPath: string | null;
  message:    string;
}

// ============================================================
// Project Discovery
// ============================================================

export interface DiscoverProjectsRequest {
  rootFolders: string[];
}

export interface DiscoverProjectsResponse {
  projects:      Project[];
  projectCount:  number;
  scanDuration:  number;        // milliseconds
  errors:        string[];
}

// ============================================================
// Onboarding
// ============================================================

export interface OnboardingState {
  isComplete:     boolean;
  user:           User | null;
  enabledTools:   AiTool[];
  selectedProjects: Project[];
  styleProfile:   StyleProfile | null;
}

// ============================================================
// App Error
// ============================================================

export interface AppError {
  code:    string;
  message: string;
  details: string | null;
}
