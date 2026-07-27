import { Injectable, signal, computed } from '@angular/core';
import type { ThemeMode } from '../core/services/theme.service';

export interface SettingsState {
  theme:         ThemeMode;
  claudePath:    string | null;
  cursorPath:    string | null;
  codexPath:     string | null;
  projectRoot:   string | null;
  openAiKey:     string | null;
  azureOpenAiKey: string | null;
  loaded:        boolean;
}

const initialState: SettingsState = {
  theme:          'system',
  claudePath:     null,
  cursorPath:     null,
  codexPath:      null,
  projectRoot:    null,
  openAiKey:      null,
  azureOpenAiKey: null,
  loaded:         false,
};

@Injectable({ providedIn: 'root' })
export class SettingsStore {
  private readonly _state = signal<SettingsState>(initialState);

  readonly state    = this._state.asReadonly();
  readonly theme    = computed(() => this._state().theme);
  readonly isLoaded = computed(() => this._state().loaded);

  readonly hasAiProvider = computed(() =>
    this._state().openAiKey !== null || this._state().azureOpenAiKey !== null,
  );

  readonly hasAnyAiTool = computed(() =>
    this._state().claudePath !== null ||
    this._state().cursorPath !== null ||
    this._state().codexPath  !== null,
  );

  setAll(settings: Omit<SettingsState, 'loaded'>): void {
    this._state.set({ ...settings, loaded: true });
  }

  patch(partial: Partial<SettingsState>): void {
    this._state.update(s => ({ ...s, ...partial }));
  }

  clear(): void {
    this._state.set(initialState);
  }
}
