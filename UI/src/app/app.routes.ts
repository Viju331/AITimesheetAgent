import { Routes } from '@angular/router';
import { requireOnboardingGuard, skipOnboardingIfCompleteGuard } from './features/onboarding/guards/onboarding.guard';

export const routes: Routes = [
  {
    path: '',
    redirectTo: 'dashboard',
    pathMatch: 'full',
  },
  {
    path: 'onboarding',
    canActivate: [skipOnboardingIfCompleteGuard],
    loadComponent: () =>
      import('./features/onboarding/pages/onboarding-page/onboarding-page.component').then(
        m => m.OnboardingPageComponent,
      ),
  },
  {
    path: 'dashboard',
    canActivate: [requireOnboardingGuard],
    loadComponent: () =>
      import('./features/dashboard/pages/dashboard-page/dashboard-page.component').then(
        m => m.DashboardPageComponent,
      ),
  },
  {
    path: 'projects',
    canActivate: [requireOnboardingGuard],
    loadComponent: () =>
      import('./features/projects/pages/projects-page/projects-page.component').then(
        m => m.ProjectsPageComponent,
      ),
  },
  {
    path: 'sessions',
    canActivate: [requireOnboardingGuard],
    loadComponent: () =>
      import('./features/sessions/pages/sessions-page/sessions-page.component').then(
        m => m.SessionsPageComponent,
      ),
  },
  {
    path: 'activities',
    canActivate: [requireOnboardingGuard],
    loadComponent: () =>
      import('./features/activities/pages/activities-page/activities-page.component').then(
        m => m.ActivitiesPageComponent,
      ),
  },
  {
    path: 'timesheets',
    canActivate: [requireOnboardingGuard],
    loadComponent: () =>
      import('./features/timesheets/pages/timesheets-page/timesheets-page.component').then(
        m => m.TimesheetsPageComponent,
      ),
  },
  {
    path: 'settings',
    loadComponent: () =>
      import('./features/settings/pages/settings-page/settings-page.component').then(
        m => m.SettingsPageComponent,
      ),
  },
  {
    path: '**',
    redirectTo: 'dashboard',
  },
];
