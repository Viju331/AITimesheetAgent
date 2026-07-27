export interface TimesheetDto {
  id: string;
  userId: string | null;
  generatedDate: string;
  content: string;
  rawDraft: string | null;
  status: 'draft' | 'final' | string;
  activityIds: string | null;
  createdAt: string;
  exportedAt: string | null;
}

export interface GenerateTimesheetResponse {
  timesheetId: string;
  taskGroupCount: number;
  bulletCount: number;
  qualityScore: QualityScoreDto;
  plainText: string;
  markdown: string;
  html: string;
  contentJson: string;
}

export interface QualityScoreDto {
  completeness: number;
  confidence: number;
  duplication: number;
  coverage: number;
  overall: number;
}

export interface TaskGroupDto {
  id: string;
  projectId: string | null;
  groupDate: string;
  title: string;
  taskKey: string;
  ticketReference: string | null;
  createdAt: string;
}

export interface FeatureGroupDto {
  title: string;
  taskTitles: string[];
}

export interface StyleProfileDto {
  preferredVerbs: string[];
  bulletChar: string;
  groupBy: string;
  verbosity: string;
  sentenceFormat: string;
  headingStyle: string;
  examples: string[];
  sourceFiles: string[];
}

/** The structured draft JSON stored in `generated_timesheets.content`. */
export interface DraftJson {
  taskGroups: DraftTaskGroup[];
}

export interface DraftTaskGroup {
  title: string;
  ticketReference: string | null;
  bullets: string[];
}
