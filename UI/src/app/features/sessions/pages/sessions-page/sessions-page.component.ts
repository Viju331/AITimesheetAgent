import { Component, inject } from '@angular/core';
import type { OnDestroy, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { SessionService } from '../../services/session.service';
import { PageHeaderComponent } from '../../../../shared/components/page-header/page-header.component';
import { EmptyStateComponent } from '../../../../shared/components/empty-state/empty-state.component';
import { LoadingSpinnerComponent } from '../../../../shared/components/loading-spinner/loading-spinner.component';

const AUTO_REFRESH_INTERVAL_MS = 5 * 60 * 1000;

@Component({
  selector: 'app-sessions-page',
  standalone: true,
  imports: [
    CommonModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatSnackBarModule,
    PageHeaderComponent,
    EmptyStateComponent,
    LoadingSpinnerComponent,
  ],
  templateUrl: './sessions-page.component.html',
  styleUrl: './sessions-page.component.scss',
})
export class SessionsPageComponent implements OnInit, OnDestroy {
  readonly sessionService = inject(SessionService);
  private readonly snackbar = inject(MatSnackBar);
  private autoRefreshHandle: ReturnType<typeof setInterval> | undefined;

  readonly dashboard = this.sessionService.dashboard;
  readonly isScanning = this.sessionService.isScanning;

  async ngOnInit(): Promise<void> {
    await this.sessionService.loadDashboard();
    this.autoRefreshHandle = setInterval(() => {
      void this.sessionService.refresh();
    }, AUTO_REFRESH_INTERVAL_MS);
  }

  ngOnDestroy(): void {
    if (this.autoRefreshHandle !== undefined) {
      clearInterval(this.autoRefreshHandle);
    }
  }

  async refresh(): Promise<void> {
    const summary = await this.sessionService.refresh();
    if (summary) {
      this.snackbar.open(
        `Scan complete — ${summary.sessionsStored} session(s), ${summary.activitiesExtracted} activity(ies)`,
        'OK',
        { duration: 3000 },
      );
    }
  }
}
