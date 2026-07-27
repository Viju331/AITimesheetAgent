import { Component, input, output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, RouterLinkActive } from '@angular/router';
import { MatListModule } from '@angular/material/list';
import { MatIconModule } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';

interface NavItem {
  label: string;
  icon:  string;
  route: string;
}

@Component({
  selector: 'app-sidebar',
  standalone: true,
  imports: [CommonModule, RouterModule, RouterLinkActive, MatListModule, MatIconModule, MatTooltipModule],
  templateUrl: './sidebar.component.html',
  styleUrl: './sidebar.component.scss',
})
export class SidebarComponent {
  readonly collapsed = input(false);

  readonly navItems: NavItem[] = [
    { label: 'Dashboard',  icon: 'dashboard',     route: '/dashboard'  },
    { label: 'Projects',   icon: 'folder',         route: '/projects'   },
    { label: 'Sessions',   icon: 'smart_toy',      route: '/sessions'   },
    { label: 'Activities', icon: 'timeline',       route: '/activities' },
    { label: 'Timesheets', icon: 'description',    route: '/timesheets' },
    { label: 'Settings',   icon: 'settings',       route: '/settings'   },
  ];
}
