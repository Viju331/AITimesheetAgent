export interface RepositoryDto {
  id: string;
  projectId: string;
  rootPath: string;
  currentBranch: string | null;
  lastCheckoutAt: string | null;
  lastScannedAt: string | null;
  createdAt: string;
}

export interface CommitDto {
  id: string;
  projectId: string;
  commitHash: string;
  branch: string | null;
  commitDate: string;
  commitMessage: string;
  authorName: string | null;
  authorEmail: string | null;
  filesChanged: number;
  insertions: number;
  deletions: number;
  changeType: string | null;
  scannedAt: string;
}

export interface FileChangeDto {
  id: string;
  commitId: string | null;
  repositoryId: string | null;
  filePath: string;
  fileName: string;
  extension: string | null;
  folder: string | null;
  changeType: string;
  category: string;
  insertions: number;
  deletions: number;
  isWorkingTree: boolean;
  detectedAt: string;
}

export interface GitDashboardDto {
  repositoryCount: number;
  commitCount: number;
  modifiedFileCount: number;
  activityCount: number;
  repositories: RepositoryDto[];
  recentCommits: CommitDto[];
}

export interface GitScanSummaryDto {
  repositoriesScanned: number;
  commitsFound: number;
  workingTreeChanges: number;
  activitiesCorrelated: number;
  activitiesCreated: number;
}

export interface ActivityDto {
  id: string;
  projectId: string | null;
  activityDate: string;
  activityType: string;
  title: string;
  description: string | null;
  sourceSessionId: string | null;
  sourceCommitId: string | null;
  confidenceScore: number;
  isFlagged: boolean;
  createdAt: string;
}

export const REFRESH_INTERVAL_OPTIONS = [
  { label: 'Manual only', minutes: 0 },
  { label: 'Every 5 minutes', minutes: 5 },
  { label: 'Every 15 minutes', minutes: 15 },
  { label: 'Every 30 minutes', minutes: 30 },
] as const;

export const GIT_REFRESH_INTERVAL_KEY = 'git.refresh_interval_minutes';
