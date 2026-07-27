import type { ThemeMode } from '../../../core/services/theme.service';

export interface AppSettings {
  theme:          ThemeMode;
  claudePath:     string | null;
  cursorPath:     string | null;
  codexPath:      string | null;
  projectRoot:    string | null;
  openAiKey:      string | null;
  azureOpenAiKey: string | null;
}

export const DEFAULT_SETTINGS: AppSettings = {
  theme:          'system',
  claudePath:     null,
  cursorPath:     null,
  codexPath:      null,
  projectRoot:    null,
  openAiKey:      null,
  azureOpenAiKey: null,
};

/** Keys stored in SQLite settings table */
export const SETTING_KEYS = {
  THEME:            'app.theme',
  CLAUDE_PATH:      'ai.claude_path',
  CURSOR_PATH:      'ai.cursor_path',
  CODEX_PATH:       'ai.codex_path',
  PROJECT_ROOT:     'projects.root_folder',
  OPENAI_KEY:       'ai.openai_api_key',
  AZURE_OPENAI_KEY: 'ai.azure_openai_api_key',
} as const;
