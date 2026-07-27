import { Component, input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';

@Component({
  selector: 'app-page-header',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatIconModule],
  template: `
    <div class="page-header">
      <div class="page-header-title">
        @if (icon()) {
          <mat-icon class="title-icon">{{ icon() }}</mat-icon>
        }
        <div>
          <h1 class="title">{{ title() }}</h1>
          @if (subtitle()) {
            <p class="subtitle">{{ subtitle() }}</p>
          }
        </div>
      </div>
      <div class="page-header-actions">
        <ng-content />
      </div>
    </div>
  `,
  styles: [`
    .page-header {
      display: flex; align-items: center;
      justify-content: space-between;
      margin-bottom: 24px;
    }
    .page-header-title {
      display: flex; align-items: center; gap: 12px;
    }
    .title-icon { font-size: 32px; width: 32px; height: 32px; color: #3949ab; }
    .title { font-size: 24px; font-weight: 500; margin: 0; }
    .subtitle { font-size: 14px; color: #9e9e9e; margin: 2px 0 0; }
    .page-header-actions { display: flex; gap: 8px; }
  `],
})
export class PageHeaderComponent {
  readonly title    = input.required<string>();
  readonly subtitle = input('');
  readonly icon     = input('');
}
