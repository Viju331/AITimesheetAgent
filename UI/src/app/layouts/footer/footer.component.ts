import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { environment } from '../../../environments/environment';

@Component({
  selector: 'app-footer',
  standalone: true,
  imports: [CommonModule],
  template: `
    <footer class="app-footer">
      <span>AI Timesheet Agent v{{ version }}</span>
      <span class="flex-1"></span>
      <span class="text-xs opacity-60">Local · Offline First</span>
    </footer>
  `,
  styles: [`
    .app-footer {
      height: var(--app-footer-height);
      display: flex;
      align-items: center;
      padding: 0 16px;
      font-size: 12px;
      border-top: 1px solid var(--color-border);
      background: #fff;
      color: #9e9e9e;
    }
    .dark-theme .app-footer {
      background: #1a1a2e;
      border-top-color: var(--color-border-dark);
    }
  `],
})
export class FooterComponent {
  readonly version = environment.version;
}
