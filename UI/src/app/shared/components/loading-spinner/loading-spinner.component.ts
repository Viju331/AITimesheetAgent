import { Component, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

@Component({
  selector: 'app-loading-spinner',
  standalone: true,
  imports: [CommonModule, MatProgressSpinnerModule],
  template: `
    <div class="spinner-wrapper" [class.overlay]="overlay()">
      <mat-spinner [diameter]="diameter()" />
      @if (message()) {
        <p class="spinner-message">{{ message() }}</p>
      }
    </div>
  `,
  styles: [`
    .spinner-wrapper {
      display: flex; flex-direction: column;
      align-items: center; justify-content: center;
      padding: 32px; gap: 12px;
    }
    .spinner-wrapper.overlay {
      position: fixed; inset: 0;
      background: rgba(0,0,0,0.3);
      z-index: 1000;
    }
    .spinner-message { color: #666; font-size: 14px; }
  `],
})
export class LoadingSpinnerComponent {
  readonly diameter = input(48);
  readonly message  = input('');
  readonly overlay  = input(false);
}
