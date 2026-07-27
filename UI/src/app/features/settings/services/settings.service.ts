import { Injectable, inject } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { LoggingService } from '../../../core/services/logging.service';
import { ErrorService } from '../../../core/errors/error.service';
import { SettingsStore } from '../../../store/settings.store';
import { ThemeService } from '../../../core/services/theme.service';
import { type AppSettings, DEFAULT_SETTINGS, SETTING_KEYS } from '../models/settings.model';

interface RawSetting { key: string; value: string; is_encrypted: boolean }

@Injectable({ providedIn: 'root' })
export class SettingsService {
  private readonly store   = inject(SettingsStore);
  private readonly theme   = inject(ThemeService);
  private readonly logger  = inject(LoggingService);
  private readonly errors  = inject(ErrorService);

  async loadAll(): Promise<void> {
    try {
      const raw = await invoke<RawSetting[]>('get_settings');
      const settings = this.mapRawToSettings(raw);
      this.store.setAll(settings);
      this.theme.setMode(settings.theme);
      this.logger.info('Settings loaded', 'SettingsService');
    } catch (err) {
      this.errors.handle(err, 'SettingsService.loadAll');
    }
  }

  async save(key: string, value: string, isEncrypted = false): Promise<void> {
    try {
      await invoke('upsert_setting', { request: { key, value, is_encrypted: isEncrypted } });
      this.logger.info(`Setting saved: ${key}`, 'SettingsService');
    } catch (err) {
      this.errors.handle(err, 'SettingsService.save');
      throw err;
    }
  }

  async saveTheme(mode: string): Promise<void> {
    await this.save(SETTING_KEYS.THEME, mode);
    this.store.patch({ theme: mode as AppSettings['theme'] });
    this.theme.setMode(mode as AppSettings['theme']);
  }

  async saveAiPaths(paths: { claudePath?: string; cursorPath?: string; codexPath?: string }): Promise<void> {
    const saves: Promise<void>[] = [];
    if (paths.claudePath !== undefined) saves.push(this.save(SETTING_KEYS.CLAUDE_PATH, paths.claudePath ?? ''));
    if (paths.cursorPath !== undefined) saves.push(this.save(SETTING_KEYS.CURSOR_PATH, paths.cursorPath ?? ''));
    if (paths.codexPath  !== undefined) saves.push(this.save(SETTING_KEYS.CODEX_PATH,  paths.codexPath  ?? ''));
    await Promise.all(saves);
    this.store.patch({
      claudePath: paths.claudePath ?? this.store.state().claudePath,
      cursorPath: paths.cursorPath ?? this.store.state().cursorPath,
      codexPath:  paths.codexPath  ?? this.store.state().codexPath,
    });
  }

  async saveApiKeys(keys: { openAiKey?: string; azureOpenAiKey?: string }): Promise<void> {
    const saves: Promise<void>[] = [];
    if (keys.openAiKey      !== undefined) saves.push(this.save(SETTING_KEYS.OPENAI_KEY,       keys.openAiKey,      true));
    if (keys.azureOpenAiKey !== undefined) saves.push(this.save(SETTING_KEYS.AZURE_OPENAI_KEY, keys.azureOpenAiKey, true));
    await Promise.all(saves);
  }

  private mapRawToSettings(raw: RawSetting[]): AppSettings {
    const map = new Map(raw.map(r => [r.key, r.value]));
    return {
      theme:          (map.get(SETTING_KEYS.THEME) as AppSettings['theme']) ?? DEFAULT_SETTINGS.theme,
      claudePath:     map.get(SETTING_KEYS.CLAUDE_PATH)      ?? null,
      cursorPath:     map.get(SETTING_KEYS.CURSOR_PATH)      ?? null,
      codexPath:      map.get(SETTING_KEYS.CODEX_PATH)       ?? null,
      projectRoot:    map.get(SETTING_KEYS.PROJECT_ROOT)     ?? null,
      openAiKey:      map.get(SETTING_KEYS.OPENAI_KEY)       ?? null,
      azureOpenAiKey: map.get(SETTING_KEYS.AZURE_OPENAI_KEY) ?? null,
    };
  }
}
