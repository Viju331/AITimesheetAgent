export interface SessionDto {
  id: string;
  projectId: string | null;
  toolId: string | null;
  sessionDate: string;
  filePath: string;
  rawSummary: string | null;
  messageCount: number;
  startedAt: string | null;
  endedAt: string | null;
  parsedAt: string;
}

export interface ActivityDto {
  id: string;
  projectId: string | null;
  activityDate: string;
  activityType: string;
  title: string;
  description: string | null;
  sourceSessionId: string | null;
  confidenceScore: number;
  isFlagged: boolean;
  createdAt: string;
}

export interface TimelineEventDto {
  id: string;
  projectId: string | null;
  sessionId: string | null;
  activityId: string | null;
  eventDate: string;
  eventTime: string;
  title: string;
  description: string | null;
}

export interface SessionDashboardDto {
  sessionCount: number;
  messageCount: number;
  activityCount: number;
  timeline: TimelineEventDto[];
}

export interface ScanSummaryDto {
  sessionsScanned: number;
  sessionsSkippedUnchanged: number;
  sessionsStored: number;
  activitiesExtracted: number;
}
