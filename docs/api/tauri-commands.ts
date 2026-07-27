/**
 * Tauri IPC command contract — Angular ↔ Rust.
 *
 * Each function maps to a #[tauri::command] in src-tauri/src/commands/.
 * Rust command names use snake_case; TypeScript wrappers use camelCase.
 *
 * Usage in Angular services:
 *   import { invoke } from '@tauri-apps/api/core';
 *   const projects = await invoke<Project[]>('get_projects');
 *
 * This file documents the full contract. Actual Angular service implementations
 * live in src/app/core/services/tauri/ with proper error handling.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  User,
  CreateUserRequest,
  Setting,
  UpsertSettingRequest,
  AiTool,
  UpdateAiToolRequest,
  Project,
  UpdateProjectSelectionRequest,
  AiSession,
  GitActivity,
  Activity,
  StyleProfile,
  GeneratedTimesheet,
  UpdateTimesheetContentRequest,
  ScanSessionsRequest,
  ScanSessionsResponse,
  AnalyzeGitRequest,
  AnalyzeGitResponse,
  GenerateTimesheetRequest,
  GenerateTimesheetResponse,
  ExportTimesheetRequest,
  ExportTimesheetResponse,
  DiscoverProjectsRequest,
  DiscoverProjectsResponse,
  OnboardingState,
} from './models';

// ============================================================
// User Commands
// ============================================================

/** rust: get_user */
export const getUser = (): Promise<User | null> =>
  invoke('get_user');

/** rust: create_user */
export const createUser = (req: CreateUserRequest): Promise<User> =>
  invoke('create_user', { request: req });

/** rust: update_user */
export const updateUser = (req: Partial<CreateUserRequest> & { id: string }): Promise<User> =>
  invoke('update_user', { request: req });

// ============================================================
// Settings Commands
// ============================================================

/** rust: get_settings */
export const getSettings = (): Promise<Setting[]> =>
  invoke('get_settings');

/** rust: get_setting */
export const getSetting = (key: string): Promise<Setting | null> =>
  invoke('get_setting', { key });

/** rust: upsert_setting */
export const upsertSetting = (req: UpsertSettingRequest): Promise<Setting> =>
  invoke('upsert_setting', { request: req });

/** rust: delete_setting */
export const deleteSetting = (key: string): Promise<void> =>
  invoke('delete_setting', { key });

// ============================================================
// AI Tool Commands
// ============================================================

/** rust: get_ai_tools — returns all registered AI tools with their current config */
export const getAiTools = (): Promise<AiTool[]> =>
  invoke('get_ai_tools');

/** rust: update_ai_tool — enable/disable and set session folder */
export const updateAiTool = (req: UpdateAiToolRequest): Promise<AiTool> =>
  invoke('update_ai_tool', { request: req });

/** rust: resolve_ai_tool_folder — detects default session folder for a given tool */
export const resolveAiToolFolder = (toolName: string): Promise<string | null> =>
  invoke('resolve_ai_tool_folder', { toolName });

// ============================================================
// Project Commands
// ============================================================

/** rust: get_projects — returns all stored projects */
export const getProjects = (): Promise<Project[]> =>
  invoke('get_projects');

/** rust: get_selected_projects — returns only projects with is_selected = true */
export const getSelectedProjects = (): Promise<Project[]> =>
  invoke('get_selected_projects');

/** rust: discover_projects — scans root folders and persists discovered projects */
export const discoverProjects = (req: DiscoverProjectsRequest): Promise<DiscoverProjectsResponse> =>
  invoke('discover_projects', { request: req });

/** rust: update_project_selection — toggle a project's is_selected flag */
export const updateProjectSelection = (req: UpdateProjectSelectionRequest): Promise<Project> =>
  invoke('update_project_selection', { request: req });

/** rust: delete_project — removes project and cascades to related records */
export const deleteProject = (id: string): Promise<void> =>
  invoke('delete_project', { id });

// ============================================================
// AI Session Commands
// ============================================================

/** rust: scan_sessions — reads AI tool session files for the given projects and date */
export const scanSessions = (req: ScanSessionsRequest): Promise<ScanSessionsResponse> =>
  invoke('scan_sessions', { request: req });

/** rust: get_sessions_by_date — returns stored sessions for a given date */
export const getSessionsByDate = (date: string): Promise<AiSession[]> =>
  invoke('get_sessions_by_date', { date });

// ============================================================
// Git Intelligence Commands
// ============================================================

/** rust: analyze_git — runs git commands against selected projects for the given date */
export const analyzeGit = (req: AnalyzeGitRequest): Promise<AnalyzeGitResponse> =>
  invoke('analyze_git', { request: req });

/** rust: get_git_activities_by_date */
export const getGitActivitiesByDate = (date: string): Promise<GitActivity[]> =>
  invoke('get_git_activities_by_date', { date });

// ============================================================
// Activity Commands
// ============================================================

/** rust: get_activities_by_date — returns normalized activities for a date */
export const getActivitiesByDate = (date: string): Promise<Activity[]> =>
  invoke('get_activities_by_date', { date });

/** rust: build_activities — merges sessions + git data into normalized activities */
export const buildActivities = (date: string, projectIds: string[]): Promise<Activity[]> =>
  invoke('build_activities', { date, projectIds });

// ============================================================
// Style Profile Commands
// ============================================================

/** rust: get_style_profile — returns the user's current style profile */
export const getStyleProfile = (): Promise<StyleProfile | null> =>
  invoke('get_style_profile');

/** rust: analyze_style_files — parses uploaded files and saves/updates the style profile */
export const analyzeStyleFiles = (filePaths: string[]): Promise<StyleProfile> =>
  invoke('analyze_style_files', { filePaths });

// ============================================================
// Timesheet Commands
// ============================================================

/** rust: generate_timesheet — calls AI provider and returns a draft timesheet */
export const generateTimesheet = (req: GenerateTimesheetRequest): Promise<GenerateTimesheetResponse> =>
  invoke('generate_timesheet', { request: req });

/** rust: get_timesheets — returns all stored timesheets, newest first */
export const getTimesheets = (): Promise<GeneratedTimesheet[]> =>
  invoke('get_timesheets');

/** rust: get_timesheet_by_date */
export const getTimesheetByDate = (date: string): Promise<GeneratedTimesheet | null> =>
  invoke('get_timesheet_by_date', { date });

/** rust: update_timesheet_content — persists user edits to a draft */
export const updateTimesheetContent = (req: UpdateTimesheetContentRequest): Promise<GeneratedTimesheet> =>
  invoke('update_timesheet_content', { request: req });

/** rust: export_timesheet — exports to clipboard or file */
export const exportTimesheet = (req: ExportTimesheetRequest): Promise<ExportTimesheetResponse> =>
  invoke('export_timesheet', { request: req });

// ============================================================
// Onboarding Commands
// ============================================================

/** rust: get_onboarding_state — returns complete onboarding status */
export const getOnboardingState = (): Promise<OnboardingState> =>
  invoke('get_onboarding_state');

/** rust: complete_onboarding — marks onboarding as done (sets 'onboarding.complete' setting) */
export const completeOnboarding = (): Promise<void> =>
  invoke('complete_onboarding');

// ============================================================
// File System Commands
// ============================================================

/** rust: pick_folder — opens OS native folder picker dialog, returns selected path */
export const pickFolder = (): Promise<string | null> =>
  invoke('pick_folder');

/** rust: pick_files — opens OS native file picker, returns selected paths */
export const pickFiles = (filters: Array<{ name: string; extensions: string[] }>): Promise<string[]> =>
  invoke('pick_files', { filters });

/** rust: path_exists — checks whether a path exists on the filesystem */
export const pathExists = (path: string): Promise<boolean> =>
  invoke('path_exists', { path });
