import { Component, inject } from '@angular/core';
import type { OnDestroy, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import { MatChipsModule } from '@angular/material/chips';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { GitService } from '../../services/git.service';
import { REFRESH_INTERVAL_OPTIONS } from '../../models/git.model';
import { PageHeaderComponent } from '../../../../shared/components/page-header/page-header.component';
import { EmptyStateComponent } from '../../../../shared/components/empty-state/empty-state.component';

@Component({
  selector: 'app-activities-page',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatFormFieldModule,
    MatSelectModule,
    MatChipsModule,
    MatSnackBarModule,
    PageHeaderComponent,
    EmptyStateComponent,
  ],
  templateUrl: './activities-page.component.html',
  styleUrl: './activities-page.component.scss',
})
export class ActivitiesPageComponent implements OnInit, OnDestroy {
  readonly gitService = inject(GitService);
  private readonly snackbar = inject(MatSnackBar);
  private autoRefreshHandle: ReturnType<typeof setInterval> | undefined;

  readonly dashboard = this.gitService.dashboard;
  readonly activities = this.gitService.activities;
  readonly isScanning = this.gitService.isScanning;
  readonly refreshIntervalMinutes = this.gitService.refreshIntervalMinutes;
  readonly intervalOptions = REFRESH_INTERVAL_OPTIONS;

  async ngOnInit(): Promise<void> {
    await Promise.all([
      this.gitService.loadDashboard(),
      this.gitService.loadActivities(),
      this.gitService.loadRefreshInterval(),
    ]);
    this.scheduleAutoRefresh();
  }

  ngOnDestroy(): void {
    this.clearAutoRefresh();
  }

  async refresh(): Promise<void> {
    const summary = await this.gitService.refresh();
    if (summary) {
      this.snackbar.open(
        `Git scan complete — ${summary.commitsFound} commit(s), ${summary.activitiesCreated} new activity(ies)`,
        'OK',
        { duration: 3000 },
      );
    }
  }

  async onIntervalChange(minutes: number): Promise<void> {
    await this.gitService.setRefreshInterval(minutes);
    this.scheduleAutoRefresh();
  }

  confidencePercent(score: number): string {
    return `${Math.round(score * 100)}%`;
  }

  private scheduleAutoRefresh(): void {
    this.clearAutoRefresh();
    const minutes = this.refreshIntervalMinutes();
    if (minutes <= 0) return;
    this.autoRefreshHandle = setInterval(() => {
      void this.gitService.refresh();
    }, minutes * 60 * 1000);
  }

  private clearAutoRefresh(): void {
    if (this.autoRefreshHandle !== undefined) {
      clearInterval(this.autoRefreshHandle);
      this.autoRefreshHandle = undefined;
    }
  }
}
