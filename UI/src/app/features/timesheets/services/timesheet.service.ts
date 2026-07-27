import { Injectable, inject, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { LoggingService } from '../../../core/services/logging.service';
import { ErrorService } from '../../../core/errors/error.service';
import type {
  DraftJson,
  FeatureGroupDto,
  GenerateTimesheetResponse,
  QualityScoreDto,
  StyleProfileDto,
  TaskGroupDto,
  TimesheetDto,
} from '../models/timesheet.model';

@Injectable({ providedIn: 'root' })
export class TimesheetService {
  private readonly logger = inject(LoggingService);
  private readonly errors = inject(ErrorService);

  readonly isGenerating = signal(false);
  readonly currentTimesheet = signal<TimesheetDto | null>(null);
  readonly currentDraft = signal<DraftJson | null>(null);
  readonly qualityScore = signal<QualityScoreDto | null>(null);
  readonly taskGroups = signal<TaskGroupDto[]>([]);
  readonly featureGroups = signal<FeatureGroupDto[]>([]);
  readonly history = signal<TimesheetDto[]>([]);
  readonly styleProfile = signal<StyleProfileDto | null>(null);
  readonly previewHtml = signal<string>('');

  async generate(date?: string): Promise<GenerateTimesheetResponse | null> {
    this.isGenerating.set(true);
    try {
      const result = await invoke<GenerateTimesheetResponse>('generate_timesheet', { date: date ?? null });
      this.qualityScore.set(result.qualityScore);
      this.previewHtml.set(result.html);
      this.logger.info(`Timesheet generated: ${result.taskGroupCount} task(s), ${result.bulletCount} bullet(s)`, 'TimesheetService');
      await this.loadCurrentDraft(date);
      await this.loadTaskGroups(date);
      return result;
    } catch (err) {
      this.errors.handle(err, 'TimesheetService.generate');
      return null;
    } finally {
      this.isGenerating.set(false);
    }
  }

  async regenerate(date: string): Promise<GenerateTimesheetResponse | null> {
    this.isGenerating.set(true);
    try {
      const result = await invoke<GenerateTimesheetResponse>('regenerate_timesheet', { date });
      this.qualityScore.set(result.qualityScore);
      this.previewHtml.set(result.html);
      await this.loadCurrentDraft(date);
      await this.loadTaskGroups(date);
      return result;
    } catch (err) {
      this.errors.handle(err, 'TimesheetService.regenerate');
      return null;
    } finally {
      this.isGenerating.set(false);
    }
  }

  async loadCurrentDraft(date?: string): Promise<void> {
    try {
      const ts = await invoke<TimesheetDto | null>('get_timesheet_by_date', { date: date ?? null });
      this.currentTimesheet.set(ts);
      if (ts?.content) {
        try {
          this.currentDraft.set(JSON.parse(ts.content));
        } catch {
          this.currentDraft.set(null);
        }
      } else {
        this.currentDraft.set(null);
      }
    } catch (err) {
      this.errors.handle(err, 'TimesheetService.loadCurrentDraft');
    }
  }

  async loadHistory(limit = 20): Promise<void> {
    try {
      const history = await invoke<TimesheetDto[]>('get_timesheet_history', { limit });
      this.history.set(history);
    } catch (err) {
      this.errors.handle(err, 'TimesheetService.loadHistory');
    }
  }

  async loadTaskGroups(date?: string): Promise<void> {
    try {
      const [groups, features] = await Promise.all([
        invoke<TaskGroupDto[]>('get_task_groups', { date: date ?? null }),
        invoke<FeatureGroupDto[]>('get_feature_groups', { date: date ?? null }),
      ]);
      this.taskGroups.set(groups);
      this.featureGroups.set(features);
    } catch (err) {
      this.errors.handle(err, 'TimesheetService.loadTaskGroups');
    }
  }

  async loadStyleProfile(): Promise<void> {
    try {
      const profile = await invoke<StyleProfileDto>('get_style_profile');
      this.styleProfile.set(profile);
    } catch (err) {
      this.errors.handle(err, 'TimesheetService.loadStyleProfile');
    }
  }

  async saveDraftContent(id: string, draftJson: DraftJson): Promise<void> {
    const content = JSON.stringify(draftJson);
    await invoke('update_timesheet_content', { id, content });
    this.currentDraft.set(draftJson);
    try {
      const html = await invoke<string>('render_timesheet', { id, format: 'html' });
      this.previewHtml.set(html);
    } catch (err) {
      this.logger.warn(String(err), 'TimesheetService.saveDraftContent.preview');
    }
  }

  async finalize(id: string): Promise<void> {
    await invoke('finalize_timesheet', { id });
    await this.loadCurrentDraft();
  }

  async submitBulletFeedback(timesheetId: string, originalText: string, editedText: string): Promise<void> {
    try {
      await invoke('submit_bullet_feedback', { timesheetId, originalText, editedText });
    } catch (err) {
      this.errors.handle(err, 'TimesheetService.submitBulletFeedback');
    }
  }

  async uploadStyleExamples(filePaths: string[]): Promise<StyleProfileDto | null> {
    try {
      const profile = await invoke<StyleProfileDto>('upload_style_examples', { filePaths });
      this.styleProfile.set(profile);
      return profile;
    } catch (err) {
      this.errors.handle(err, 'TimesheetService.uploadStyleExamples');
      return null;
    }
  }

  qualityPercent(score: QualityScoreDto): number {
    return Math.round(score.overall * 100);
  }
}
