import { Injectable, inject, signal } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { LoggingService } from '../../../core/services/logging.service';
import { ErrorService } from '../../../core/errors/error.service';
import type { AiToolDto, OnboardingStateDto, ProjectDto, UserDto } from '../models/onboarding.model';

interface RawSetting {
  key: string;
  value: string;
  is_encrypted: boolean;
}

const ROOT_FOLDER_KEY = 'projects.root_folder';

@Injectable({ providedIn: 'root' })
export class OnboardingService {
  private readonly logger = inject(LoggingService);
  private readonly errors = inject(ErrorService);

  readonly user = signal<UserDto | null>(null);
  readonly aiTools = signal<AiToolDto[]>([]);
  readonly projects = signal<ProjectDto[]>([]);
  readonly rootFolder = signal<string | null>(null);
  readonly state = signal<OnboardingStateDto>({
    hasUser: false,
    hasAiTool: false,
    hasProjects: false,
    isComplete: false,
  });
  readonly isScanning = signal(false);

  async loadState(): Promise<OnboardingStateDto> {
    try {
      const s = await invoke<OnboardingStateDto>('get_onboarding_state');
      this.state.set(s);
      return s;
    } catch (err) {
      this.errors.handle(err, 'OnboardingService.loadState');
      return this.state();
    }
  }

  async loadUser(): Promise<void> {
    try {
      const user = await invoke<UserDto | null>('get_user');
      this.user.set(user);
    } catch (err) {
      this.errors.handle(err, 'OnboardingService.loadUser');
    }
  }

  async saveUser(name: string, email: string | null): Promise<UserDto> {
    const user = await invoke<UserDto>('save_user', { name, email });
    this.user.set(user);
    this.logger.info('User profile saved', 'OnboardingService');
    return user;
  }

  async loadAiTools(): Promise<void> {
    try {
      const tools = await invoke<AiToolDto[]>('get_ai_tools');
      this.aiTools.set(tools);
    } catch (err) {
      this.errors.handle(err, 'OnboardingService.loadAiTools');
    }
  }

  async updateAiTool(id: string, isEnabled: boolean, sessionFolder: string | null): Promise<void> {
    const updated = await invoke<AiToolDto>('update_ai_tool', { id, isEnabled, sessionFolder });
    this.aiTools.update(tools => tools.map(t => (t.id === id ? updated : t)));
  }

  async pickFolder(): Promise<string | null> {
    return invoke<string | null>('pick_folder');
  }

  async loadRootFolder(): Promise<void> {
    try {
      const settings = await invoke<RawSetting[]>('get_settings');
      const setting = settings.find(s => s.key === ROOT_FOLDER_KEY);
      this.rootFolder.set(setting?.value ?? null);
    } catch (err) {
      this.errors.handle(err, 'OnboardingService.loadRootFolder');
    }
  }

  async saveRootFolder(path: string): Promise<void> {
    await invoke('upsert_setting', { request: { key: ROOT_FOLDER_KEY, value: path, is_encrypted: false } });
    this.rootFolder.set(path);
  }

  async discoverProjects(rootPaths: string[]): Promise<ProjectDto[]> {
    this.isScanning.set(true);
    try {
      const projects = await invoke<ProjectDto[]>('discover_projects', { rootPaths });
      this.projects.set(projects);
      this.logger.info(`Discovered ${projects.length} project(s)`, 'OnboardingService');
      return projects;
    } catch (err) {
      this.errors.handle(err, 'OnboardingService.discoverProjects');
      return [];
    } finally {
      this.isScanning.set(false);
    }
  }

  async loadProjects(): Promise<void> {
    try {
      const projects = await invoke<ProjectDto[]>('get_projects');
      this.projects.set(projects);
    } catch (err) {
      this.errors.handle(err, 'OnboardingService.loadProjects');
    }
  }

  async setProjectSelection(id: string, isSelected: boolean): Promise<void> {
    await invoke('set_project_selection', { id, isSelected });
    this.projects.update(ps => ps.map(p => (p.id === id ? { ...p, isSelected } : p)));
  }

  async setAllProjectsSelection(isSelected: boolean): Promise<void> {
    const ids = this.projects().map(p => p.id);
    await Promise.all(ids.map(id => invoke('set_project_selection', { id, isSelected })));
    this.projects.update(ps => ps.map(p => ({ ...p, isSelected })));
  }

  async completeOnboarding(): Promise<void> {
    await invoke('complete_onboarding');
    this.state.update(s => ({ ...s, isComplete: true }));
    this.logger.info('Onboarding completed', 'OnboardingService');
  }
}
