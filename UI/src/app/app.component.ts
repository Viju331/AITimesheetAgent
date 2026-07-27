import { Component, inject, OnInit, computed } from '@angular/core';
import { NavigationEnd, Router, RouterOutlet } from '@angular/router';
import { toSignal } from '@angular/core/rxjs-interop';
import { filter, map } from 'rxjs/operators';
import { AppShellComponent } from './layouts/app-shell/app-shell.component';
import { SettingsService } from './features/settings/services/settings.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, AppShellComponent],
  template: `
    @if (isOnboarding()) {
      <router-outlet />
    } @else {
      <app-shell />
    }
  `,
})
export class AppComponent implements OnInit {
  private readonly settingsService = inject(SettingsService);
  private readonly router = inject(Router);

  private readonly currentUrl = toSignal(
    this.router.events.pipe(
      filter((e): e is NavigationEnd => e instanceof NavigationEnd),
      map(e => e.urlAfterRedirects),
    ),
    { initialValue: this.router.url },
  );

  readonly isOnboarding = computed(() => this.currentUrl().startsWith('/onboarding'));

  async ngOnInit(): Promise<void> {
    // Load persisted settings on startup (applies theme, populates store)
    await this.settingsService.loadAll();
  }
}
