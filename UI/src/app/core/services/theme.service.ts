import { Injectable, signal, effect } from '@angular/core';

export type ThemeMode = 'light' | 'dark' | 'system';

const THEME_KEY = 'app.theme';

@Injectable({ providedIn: 'root' })
export class ThemeService {
  readonly mode = signal<ThemeMode>(this.loadSavedMode());

  /** true when the effective theme is dark (system dark or explicit dark) */
  readonly isDark = signal<boolean>(false);

  private readonly systemDarkQuery = window.matchMedia('(prefers-color-scheme: dark)');

  constructor() {
    // Apply theme on mode change and track isDark
    effect(() => {
      const m = this.mode();
      const dark = m === 'dark' || (m === 'system' && this.systemDarkQuery.matches);
      this.isDark.set(dark);
      this.applyTheme(dark);
      localStorage.setItem(THEME_KEY, m);
    });

    // React to OS theme changes when in system mode
    this.systemDarkQuery.addEventListener('change', e => {
      if (this.mode() === 'system') {
        this.isDark.set(e.matches);
        this.applyTheme(e.matches);
      }
    });
  }

  setMode(mode: ThemeMode): void {
    this.mode.set(mode);
  }

  private applyTheme(dark: boolean): void {
    document.body.classList.toggle('dark-theme', dark);
  }

  private loadSavedMode(): ThemeMode {
    const saved = localStorage.getItem(THEME_KEY) as ThemeMode | null;
    return saved ?? 'system';
  }
}
