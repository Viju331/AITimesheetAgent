import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { invoke } from '@tauri-apps/api/core';
import { SessionService } from './session.service';
import type { ScanSummaryDto, SessionDashboardDto } from '../models/session.model';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('SessionService', () => {
  let service: SessionService;
  const invokeMock = invoke as unknown as ReturnType<typeof vi.fn>;

  beforeEach(() => {
    invokeMock.mockReset();
    TestBed.configureTestingModule({});
    service = TestBed.inject(SessionService);
  });

  it('starts with an empty dashboard', () => {
    expect(service.dashboard()).toEqual({
      sessionCount: 0,
      messageCount: 0,
      activityCount: 0,
      timeline: [],
    });
  });

  it('loadDashboard populates the dashboard signal', async () => {
    const dashboard: SessionDashboardDto = {
      sessionCount: 2,
      messageCount: 10,
      activityCount: 4,
      timeline: [
        { id: 't1', projectId: 'p1', sessionId: 's1', activityId: 'a1', eventDate: '2026-06-15', eventTime: '09:00', title: 'Implemented Login', description: null },
      ],
    };
    invokeMock.mockResolvedValueOnce(dashboard);

    await service.loadDashboard();

    expect(invokeMock).toHaveBeenCalledWith('get_session_dashboard', { date: null });
    expect(service.dashboard()).toEqual(dashboard);
  });

  it('refresh triggers a scan then reloads the dashboard', async () => {
    const summary: ScanSummaryDto = {
      sessionsScanned: 3,
      sessionsSkippedUnchanged: 1,
      sessionsStored: 2,
      activitiesExtracted: 5,
    };
    const dashboard: SessionDashboardDto = { sessionCount: 2, messageCount: 5, activityCount: 5, timeline: [] };

    invokeMock.mockImplementation((command: string) => {
      if (command === 'scan_sessions') return Promise.resolve(summary);
      if (command === 'get_session_dashboard') return Promise.resolve(dashboard);
      return Promise.reject(new Error(`unexpected command ${command}`));
    });

    const result = await service.refresh();

    expect(result).toEqual(summary);
    expect(service.lastScanSummary()).toEqual(summary);
    expect(service.dashboard()).toEqual(dashboard);
    expect(service.isScanning()).toBe(false);
  });

  it('refresh returns null and surfaces an error when the scan fails', async () => {
    invokeMock.mockRejectedValueOnce(new Error('scan failed'));

    const result = await service.refresh();

    expect(result).toBeNull();
    expect(service.isScanning()).toBe(false);
  });
});
