import { Injectable, inject, signal } from '@angular/core';
import { type AppError, type ErrorCategory, makeError, ErrorCodes } from './error-types';
import { LoggingService } from '../services/logging.service';

@Injectable({ providedIn: 'root' })
export class ErrorService {
  private readonly logger = inject(LoggingService);
  private readonly _lastError = signal<AppError | null>(null);
  readonly lastError = this._lastError.asReadonly();

  handle(err: unknown, context?: string): AppError {
    const appError = this.normalize(err);
    this._lastError.set(appError);
    this.logger.error(`${appError.code}: ${appError.message}`, context ?? 'ErrorService');
    return appError;
  }

  clear(): void {
    this._lastError.set(null);
  }

  /** Convert any thrown value to an AppError */
  private normalize(err: unknown): AppError {
    if (this.isAppError(err)) return err;

    if (typeof err === 'string') {
      return makeError(ErrorCodes.UNEXPECTED, err, 'unknown');
    }

    if (err instanceof Error) {
      const category = this.categorize(err.message);
      return makeError(ErrorCodes.UNEXPECTED, err.message, category, err.stack);
    }

    if (typeof err === 'object' && err !== null && 'message' in err) {
      const msg = String((err as Record<string, unknown>)['message']);
      return makeError(ErrorCodes.UNEXPECTED, msg, 'unknown');
    }

    return makeError(ErrorCodes.UNEXPECTED, 'An unexpected error occurred.', 'unknown');
  }

  private isAppError(err: unknown): err is AppError {
    return (
      typeof err === 'object' &&
      err !== null &&
      'code'      in err &&
      'message'   in err &&
      'category'  in err &&
      'timestamp' in err
    );
  }

  private categorize(message: string): ErrorCategory {
    const lower = message.toLowerCase();
    if (lower.includes('database') || lower.includes('sqlite')) return 'database';
    if (lower.includes('network') || lower.includes('http'))    return 'network';
    if (lower.includes('file') || lower.includes('permission')) return 'system';
    return 'unknown';
  }
}
