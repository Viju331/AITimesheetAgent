export type ErrorCategory =
  | 'validation'
  | 'business'
  | 'system'
  | 'database'
  | 'network'
  | 'unknown';

export interface AppError {
  code:      string;
  message:   string;
  category:  ErrorCategory;
  details?:  string;
  timestamp: string;
}

export function makeError(
  code: string,
  message: string,
  category: ErrorCategory,
  details?: string,
): AppError {
  return { code, message, category, details, timestamp: new Date().toISOString() };
}

// Well-known error codes
export const ErrorCodes = {
  // Validation
  REQUIRED_FIELD:    'REQUIRED_FIELD',
  INVALID_PATH:      'INVALID_PATH',

  // Business
  NO_SESSIONS_FOUND: 'NO_SESSIONS_FOUND',
  NO_GIT_REPO:       'NO_GIT_REPO',
  NO_AI_TOOL:        'NO_AI_TOOL',

  // System
  FILE_READ_FAILED:  'FILE_READ_FAILED',
  COMMAND_FAILED:    'COMMAND_FAILED',

  // Database
  DB_INIT_FAILED:    'DB_INIT_FAILED',
  DB_QUERY_FAILED:   'DB_QUERY_FAILED',

  // Unknown
  UNEXPECTED:        'UNEXPECTED',
} as const;
