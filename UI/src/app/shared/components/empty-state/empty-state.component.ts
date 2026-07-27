import { Component, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'app-empty-state',
  standalone: true,
  imports: [CommonModule, MatIconModule, MatButtonModule],
  template: `
    <div class="empty-state">
      <mat-icon class="empty-icon">{{ icon() }}</mat-icon>
      <h3 class="empty-title">{{ title() }}</h3>
      @if (subtitle()) {
        <p class="empty-subtitle">{{ subtitle() }}</p>
      }
      <ng-content />
    </div>
  `,
  styles: [`
    .empty-state {
      display: flex; flex-direction: column;
      align-items: center; justify-content: center;
      padding: 48px 24px; text-align: center; gap: 8px;
    }
    .empty-icon { font-size: 64px; width: 64px; height: 64px; color: #bdbdbd; }
    .empty-title { font-size: 18px; font-weight: 500; color: #424242; margin: 8px 0 0; }
    .empty-subtitle { color: #9e9e9e; font-size: 14px; margin: 0; }
  `],
})
export class EmptyStateComponent {
  readonly icon     = input('inbox');
  readonly title    = input('Nothing here yet');
  readonly subtitle = input('');
}
