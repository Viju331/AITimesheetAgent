import { Injectable, inject, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { LoggingService } from '../../../core/services/logging.service';
import { ErrorService } from '../../../core/errors/error.service';
import {
  GIT_REFRESH_INTERVAL_KEY,
  type ActivityDto,
  type GitDashboardDto,
  type GitScanSummaryDto,
} from '../models/git.model';

interface RawSetting {
  key: string;
  value: string;
  is_encrypted: boolean;
}

@Injectable({ providedIn: 'root' })
export class GitService {
  private readonly logger = inject(LoggingService);
  private readonly errors = inject(ErrorService);

  readonly dashboard = signal<GitDashboardDto>({
    repositoryCount: 0,
    commitCount: 0,
    modifiedFileCount: 0,
    activityCount: 0,
    repositories: [],
    recentCommits: [],
  });
  readonly activities = signal<ActivityDto[]>([]);
  readonly isScanning = signal(false);
  readonly lastScanSummary = signal<GitScanSummaryDto | null>(null);
  readonly refreshIntervalMinutes = signal(5);

  async loadDashboard(date?: string): Promise<void> {
    try {
      const dashboard = await invoke<GitDashboardDto>('get_git_dashboard', { date: date ?? null });
      this.dashboard.set(dashboard);
    } catch (err) {
      this.errors.handle(err, 'GitService.loadDashboard');
    }
  }

  async loadActivities(date?: string): Promise<void> {
    try {
      const activities = await invoke<ActivityDto[]>('get_activities', { date: date ?? null });
      this.activities.set(activities);
    } catch (err) {
      this.errors.handle(err, 'GitService.loadActivities');
    }
  }

  async refresh(date?: string): Promise<GitScanSummaryDto | null> {
    this.isScanning.set(true);
    try {
      const summary = await invoke<GitScanSummaryDto>('scan_git');
      this.lastScanSummary.set(summary);
      this.logger.info(
        `Git scan complete: ${summary.commitsFound} commit(s), ${summary.activitiesCorrelated} correlated, ${summary.activitiesCreated} created`,
        'GitService',
      );
      await Promise.all([this.loadDashboard(date), this.loadActivities(date)]);
      return summary;
    } catch (err) {
      this.errors.handle(err, 'GitService.refresh');
      return null;
    } finally {
      this.isScanning.set(false);
    }
  }

  async loadRefreshInterval(): Promise<void> {
    try {
      const settings = await invoke<RawSetting[]>('get_settings');
      const setting = settings.find(s => s.key === GIT_REFRESH_INTERVAL_KEY);
      const minutes = setting ? parseInt(setting.value, 10) : 5;
      this.refreshIntervalMinutes.set(Number.isNaN(minutes) ? 5 : minutes);
    } catch (err) {
      this.errors.handle(err, 'GitService.loadRefreshInterval');
    }
  }

  async setRefreshInterval(minutes: number): Promise<void> {
    await invoke('upsert_setting', {
      request: { key: GIT_REFRESH_INTERVAL_KEY, value: String(minutes), is_encrypted: false },
    });
    this.refreshIntervalMinutes.set(minutes);
  }
}
