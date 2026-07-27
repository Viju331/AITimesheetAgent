import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { invoke } from '@tauri-apps/api/core';
import { GitService } from './git.service';
import type { GitDashboardDto, GitScanSummaryDto } from '../models/git.model';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('GitService', () => {
  let service: GitService;
  const invokeMock = invoke as unknown as ReturnType<typeof vi.fn>;

  beforeEach(() => {
    invokeMock.mockReset();
    TestBed.configureTestingModule({});
    service = TestBed.inject(GitService);
  });

  it('starts with an empty dashboard and default refresh interval', () => {
    expect(service.dashboard()).toEqual({
      repositoryCount: 0,
      commitCount: 0,
      modifiedFileCount: 0,
      activityCount: 0,
      repositories: [],
      recentCommits: [],
    });
    expect(service.refreshIntervalMinutes()).toBe(5);
  });

  it('loadDashboard populates the dashboard signal', async () => {
    const dashboard: GitDashboardDto = {
      repositoryCount: 1,
      commitCount: 2,
      modifiedFileCount: 3,
      activityCount: 1,
      repositories: [],
      recentCommits: [],
    };
    invokeMock.mockResolvedValueOnce(dashboard);

    await service.loadDashboard();

    expect(invokeMock).toHaveBeenCalledWith('get_git_dashboard', { date: null });
    expect(service.dashboard()).toEqual(dashboard);
  });

  it('refresh triggers a scan then reloads dashboard and activities', async () => {
    const summary: GitScanSummaryDto = {
      repositoriesScanned: 1,
      commitsFound: 2,
      workingTreeChanges: 0,
      activitiesCorrelated: 1,
      activitiesCreated: 1,
    };
    const dashboard: GitDashboardDto = {
      repositoryCount: 1,
      commitCount: 2,
      modifiedFileCount: 3,
      activityCount: 2,
      repositories: [],
      recentCommits: [],
    };

    invokeMock.mockImplementation((command: string) => {
      if (command === 'scan_git') return Promise.resolve(summary);
      if (command === 'get_git_dashboard') return Promise.resolve(dashboard);
      if (command === 'get_activities') return Promise.resolve([]);
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    const result = await service.refresh();

    expect(result).toEqual(summary);
    expect(service.lastScanSummary()).toEqual(summary);
    expect(service.dashboard()).toEqual(dashboard);
    expect(service.isScanning()).toBe(false);
  });

  it('refresh returns null when the scan fails', async () => {
    invokeMock.mockRejectedValueOnce(new Error('scan failed'));
    const result = await service.refresh();
    expect(result).toBeNull();
    expect(service.isScanning()).toBe(false);
  });

  it('setRefreshInterval persists the setting and updates the signal', async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await service.setRefreshInterval(15);

    expect(invokeMock).toHaveBeenCalledWith('upsert_setting', {
      request: { key: 'git.refresh_interval_minutes', value: '15', is_encrypted: false },
    });
    expect(service.refreshIntervalMinutes()).toBe(15);
  });
});
