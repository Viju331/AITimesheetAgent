import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogEntry {
  level:     LogLevel;
  message:   string;
  context?:  string;
  timestamp: string;
}

@Injectable({ providedIn: 'root' })
export class LoggingService {
  private readonly isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

  debug(message: string, context?: string): void {
    this.log('debug', message, context);
  }

  info(message: string, context?: string): void {
    this.log('info', message, context);
  }

  warn(message: string, context?: string): void {
    this.log('warn', message, context);
  }

  error(message: string, context?: string): void {
    this.log('error', message, context);
  }

  private log(level: LogLevel, message: string, context?: string): void {
    const entry: LogEntry = {
      level,
      message,
      context,
      timestamp: new Date().toISOString(),
    };

    // Always log to browser console (dev convenience)
    const prefix = context ? `[${context}]` : '';
    const formatted = `${entry.timestamp} ${level.toUpperCase()} ${prefix} ${message}`;
    switch (level) {
      case 'debug': console.debug(formatted); break;
      case 'info':  console.info(formatted);  break;
      case 'warn':  console.warn(formatted);  break;
      case 'error': console.error(formatted); break;
    }

    // Forward to Rust logging layer when running inside Tauri
    if (this.isTauri) {
      invoke('log_message', { level, message, context: context ?? '' }).catch(() => {
        // Silently ignore — avoid recursive error loop
      });
    }
  }
}
