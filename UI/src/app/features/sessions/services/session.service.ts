import { Injectable, inject, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { LoggingService } from '../../../core/services/logging.service';
import { ErrorService } from '../../../core/errors/error.service';
import type { ScanSummaryDto, SessionDashboardDto } from '../models/session.model';

@Injectable({ providedIn: 'root' })
export class SessionService {
  private readonly logger = inject(LoggingService);
  private readonly errors = inject(ErrorService);

  readonly dashboard = signal<SessionDashboardDto>({
    sessionCount: 0,
    messageCount: 0,
    activityCount: 0,
    timeline: [],
  });
  readonly isScanning = signal(false);
  readonly lastScanSummary = signal<ScanSummaryDto | null>(null);

  async loadDashboard(date?: string): Promise<void> {
    try {
      const dashboard = await invoke<SessionDashboardDto>('get_session_dashboard', { date: date ?? null });
      this.dashboard.set(dashboard);
    } catch (err) {
      this.errors.handle(err, 'SessionService.loadDashboard');
    }
  }

  async refresh(date?: string): Promise<ScanSummaryDto | null> {
    this.isScanning.set(true);
    try {
      const summary = await invoke<ScanSummaryDto>('scan_sessions');
      this.lastScanSummary.set(summary);
      this.logger.info(
        `Scan complete: ${summary.sessionsStored} stored, ${summary.activitiesExtracted} activities`,
        'SessionService',
      );
      await this.loadDashboard(date);
      return summary;
    } catch (err) {
      this.errors.handle(err, 'SessionService.refresh');
      return null;
    } finally {
      this.isScanning.set(false);
    }
  }
}
