import { describe, it, expect, beforeEach, vi } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { invoke } from '@tauri-apps/api/core';
import { TimesheetService } from './timesheet.service';
import type { DraftJson, GenerateTimesheetResponse } from '../models/timesheet.model';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

describe('TimesheetService', () => {
  let service: TimesheetService;
  const invokeMock = invoke as unknown as ReturnType<typeof vi.fn>;

  beforeEach(() => {
    invokeMock.mockReset();
    TestBed.configureTestingModule({});
    service = TestBed.inject(TimesheetService);
  });

  it('starts with empty state', () => {
    expect(service.currentTimesheet()).toBeNull();
    expect(service.qualityScore()).toBeNull();
    expect(service.isGenerating()).toBe(false);
  });

  it('generate invokes generate_timesheet and updates quality signal', async () => {
    const response: GenerateTimesheetResponse = {
      timesheetId: 'ts-1',
      taskGroupCount: 2,
      bulletCount: 4,
      qualityScore: { completeness: 1.0, confidence: 0.8, duplication: 0.9, coverage: 1.0, overall: 0.93 },
      plainText: 'Task - Admin\n- Fixed dropdown',
      markdown: '## Admin\n- Fixed dropdown',
      html: '<h3>Admin</h3><ul><li>Fixed dropdown</li></ul>',
      contentJson: JSON.stringify({ taskGroups: [{ title: 'Admin', ticketReference: null, bullets: ['Fixed dropdown'] }] }),
    };
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === 'generate_timesheet') return Promise.resolve(response);
      if (cmd === 'get_timesheet_by_date') return Promise.resolve({ id: 'ts-1', content: response.contentJson, status: 'draft', generatedDate: '2026-06-15', createdAt: '', rawDraft: null, activityIds: null, userId: null, exportedAt: null });
      if (cmd === 'get_task_groups') return Promise.resolve([]);
      if (cmd === 'get_feature_groups') return Promise.resolve([]);
      return Promise.resolve(null);
    });

    const result = await service.generate();

    expect(result).not.toBeNull();
    expect(service.qualityScore()?.overall).toBeCloseTo(0.93);
    expect(service.isGenerating()).toBe(false);
    expect(invokeMock).toHaveBeenCalledWith('generate_timesheet', { date: null });
  });

  it('generate returns null when the pipeline fails', async () => {
    invokeMock.mockRejectedValueOnce(new Error('no activities'));
    const result = await service.generate();
    expect(result).toBeNull();
    expect(service.isGenerating()).toBe(false);
  });

  it('saveDraftContent serializes the draft and invokes update', async () => {
    const draft: DraftJson = { taskGroups: [{ title: 'Admin', ticketReference: null, bullets: ['Fixed dropdown'] }] };
    invokeMock.mockResolvedValueOnce(undefined);
    invokeMock.mockResolvedValueOnce('<h3>Admin</h3>');

    await service.saveDraftContent('ts-1', draft);

    expect(invokeMock).toHaveBeenCalledWith('update_timesheet_content', {
      id: 'ts-1',
      content: JSON.stringify(draft),
    });
    expect(service.currentDraft()).toEqual(draft);
  });

  it('qualityPercent helper computes the rounded percentage', () => {
    const pct = service.qualityPercent({ completeness: 1.0, confidence: 0.8, duplication: 0.9, coverage: 1.0, overall: 0.93 });
    expect(pct).toBe(93);
  });
});
