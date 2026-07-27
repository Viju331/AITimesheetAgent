import { Component, input, output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'app-error-banner',
  standalone: true,
  imports: [CommonModule, MatIconModule, MatButtonModule],
  template: `
    @if (message()) {
      <div class="error-banner" role="alert">
        <mat-icon class="error-icon">error_outline</mat-icon>
        <span class="error-message">{{ message() }}</span>
        <button mat-icon-button (click)="dismissed.emit()" aria-label="Dismiss">
          <mat-icon>close</mat-icon>
        </button>
      </div>
    }
  `,
  styles: [`
    .error-banner {
      display: flex; align-items: center; gap: 8px;
      padding: 12px 16px;
      background: #fdecea; border: 1px solid #f44336;
      border-radius: var(--app-border-radius, 8px);
      color: #b71c1c; margin-bottom: 16px;
    }
    .error-icon { color: #f44336; }
    .error-message { flex: 1; font-size: 14px; }
  `],
})
export class ErrorBannerComponent {
  readonly message  = input('');
  readonly dismissed = output<void>();
}
