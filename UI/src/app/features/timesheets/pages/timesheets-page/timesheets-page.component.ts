import { Component, computed, inject, signal } from '@angular/core';
import type { OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatTabsModule } from '@angular/material/tabs';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatInputModule } from '@angular/material/input';
import { MatChipsModule } from '@angular/material/chips';
import { MatTooltipModule } from '@angular/material/tooltip';
import { MatDividerModule } from '@angular/material/divider';
import { MatListModule } from '@angular/material/list';
import { DragDropModule, moveItemInArray } from '@angular/cdk/drag-drop';
import type { CdkDragDrop } from '@angular/cdk/drag-drop';
import { invoke } from '@tauri-apps/api/core';
import { TimesheetService } from '../../services/timesheet.service';
import { PageHeaderComponent } from '../../../../shared/components/page-header/page-header.component';
import { EmptyStateComponent } from '../../../../shared/components/empty-state/empty-state.component';
import type { DraftJson, DraftTaskGroup } from '../../models/timesheet.model';

@Component({
  selector: 'app-timesheets-page',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatTabsModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatProgressBarModule,
    MatProgressSpinnerModule,
    MatSnackBarModule,
    MatInputModule,
    MatChipsModule,
    MatTooltipModule,
    MatDividerModule,
    MatListModule,
    DragDropModule,
    PageHeaderComponent,
    EmptyStateComponent,
  ],
  templateUrl: './timesheets-page.component.html',
  styleUrl: './timesheets-page.component.scss',
})
export class TimesheetsPageComponent implements OnInit {
  readonly ts = inject(TimesheetService);
  private readonly snackbar = inject(MatSnackBar);

  readonly editableDraft = signal<DraftJson | null>(null);
  readonly previewFormat = signal<'html' | 'markdown' | 'plain'>('html');
  readonly previewContent = signal<string>('');

  readonly qualityPercent = computed(() => {
    const q = this.ts.qualityScore();
    return q ? Math.round(q.overall * 100) : 0;
  });

  async ngOnInit(): Promise<void> {
    await Promise.all([
      this.ts.loadCurrentDraft(),
      this.ts.loadTaskGroups(),
      this.ts.loadHistory(),
      this.ts.loadStyleProfile(),
    ]);
    this.syncEditableDraft();
  }

  async generate(): Promise<void> {
    const result = await this.ts.generate();
    if (result) {
      this.syncEditableDraft();
      this.snackbar.open(
        `Generated: ${result.taskGroupCount} task(s), ${result.bulletCount} bullet(s) — ${Math.round(result.qualityScore.overall * 100)}% quality`,
        'OK',
        { duration: 4000 },
      );
    }
  }

  async regenerate(): Promise<void> {
    const current = this.ts.currentTimesheet();
    if (!current) return;
    const result = await this.ts.regenerate(current.generatedDate);
    if (result) {
      this.syncEditableDraft();
      this.snackbar.open('Regenerated fresh draft', 'OK', { duration: 2500 });
    }
  }

  async saveEdits(): Promise<void> {
    const id = this.ts.currentTimesheet()?.id;
    const draft = this.editableDraft();
    if (!id || !draft) return;
    await this.ts.saveDraftContent(id, draft);
    this.snackbar.open('Draft saved', 'OK', { duration: 2000 });
  }

  async finalize(): Promise<void> {
    const id = this.ts.currentTimesheet()?.id;
    if (!id) return;
    await this.ts.finalize(id);
    this.snackbar.open('Timesheet finalized', 'OK', { duration: 2500 });
  }

  async switchPreview(format: 'html' | 'markdown' | 'plain'): Promise<void> {
    this.previewFormat.set(format);
    const id = this.ts.currentTimesheet()?.id;
    if (!id) {
      this.previewContent.set('');
      return;
    }
    const text = await invoke<string>('render_timesheet', { id, format });
    this.previewContent.set(text);
  }

  async onBulletBlur(groupIndex: number, bulletIndex: number, originalText: string): Promise<void> {
    const draft = this.editableDraft();
    const id = this.ts.currentTimesheet()?.id;
    if (!draft || !id) return;
    const currentText = draft.taskGroups[groupIndex].bullets[bulletIndex];
    if (currentText !== originalText) {
      await this.ts.submitBulletFeedback(id, originalText, currentText);
    }
  }

  addBullet(groupIndex: number): void {
    this.editableDraft.update(d => {
      if (!d) return d;
      const copy = structuredClone(d);
      copy.taskGroups[groupIndex].bullets.push('');
      return copy;
    });
  }

  removeBullet(groupIndex: number, bulletIndex: number): void {
    this.editableDraft.update(d => {
      if (!d) return d;
      const copy = structuredClone(d);
      copy.taskGroups[groupIndex].bullets.splice(bulletIndex, 1);
      return copy;
    });
  }

  dropBullet(groupIndex: number, event: CdkDragDrop<string[]>): void {
    this.editableDraft.update(d => {
      if (!d) return d;
      const copy = structuredClone(d);
      moveItemInArray(copy.taskGroups[groupIndex].bullets, event.previousIndex, event.currentIndex);
      return copy;
    });
  }

  addTaskGroup(): void {
    this.editableDraft.update(d => {
      const copy: DraftJson = d ? structuredClone(d) : { taskGroups: [] };
      copy.taskGroups.push({ title: 'New Task', ticketReference: null, bullets: [''] });
      return copy;
    });
  }

  removeTaskGroup(index: number): void {
    this.editableDraft.update(d => {
      if (!d) return d;
      const copy = structuredClone(d);
      copy.taskGroups.splice(index, 1);
      return copy;
    });
  }

  trackGroup(_: number, group: DraftTaskGroup): string {
    return group.title;
  }

  async uploadStyleFiles(): Promise<void> {
    const path = await invoke<string | null>('pick_folder');
    if (!path) return;
    const result = await this.ts.uploadStyleExamples([path]);
    if (result) {
      this.snackbar.open(
        `Style updated from ${result.sourceFiles.length} file(s) — verbs: ${result.preferredVerbs.slice(0, 3).join(', ')}`,
        'OK',
        { duration: 4000 },
      );
    }
  }

  private syncEditableDraft(): void {
    const draft = this.ts.currentDraft();
    this.editableDraft.set(draft ? structuredClone(draft) : null);
  }
}
