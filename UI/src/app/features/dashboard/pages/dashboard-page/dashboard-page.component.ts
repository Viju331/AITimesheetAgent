import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-dashboard-page',
  standalone: true,
  imports: [CommonModule],
  template: `<div class="page-placeholder"><h2>Dashboard</h2></div>`,
  styles: [`
    .page-placeholder { padding: 24px; }
    h2 { color: #666; font-weight: 400; }
  `]
})
export class DashboardPageComponent {}
