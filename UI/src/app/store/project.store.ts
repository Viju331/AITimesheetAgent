import { Injectable, signal, computed } from '@angular/core';

export interface Project {
  id:          string;
  name:        string;
  path:        string;
  techStack:   string;
  hasGit:      boolean;
  isSelected:  boolean;
  lastScanned: string | null;
}

@Injectable({ providedIn: 'root' })
export class ProjectStore {
  private readonly _projects = signal<Project[]>([]);
  private readonly _loading  = signal(false);
  private readonly _error    = signal<string | null>(null);

  readonly projects        = this._projects.asReadonly();
  readonly loading         = this._loading.asReadonly();
  readonly error           = this._error.asReadonly();

  readonly selectedProjects = computed(() => this._projects().filter(p => p.isSelected));
  readonly projectCount     = computed(() => this._projects().length);
  readonly hasProjects      = computed(() => this._projects().length > 0);

  setProjects(projects: Project[]): void {
    this._projects.set(projects);
  }

  setLoading(loading: boolean): void {
    this._loading.set(loading);
  }

  setError(error: string | null): void {
    this._error.set(error);
  }

  toggleSelection(id: string): void {
    this._projects.update(list =>
      list.map(p => p.id === id ? { ...p, isSelected: !p.isSelected } : p),
    );
  }

  upsert(project: Project): void {
    this._projects.update(list => {
      const idx = list.findIndex(p => p.id === project.id);
      return idx >= 0
        ? list.map((p, i) => i === idx ? project : p)
        : [...list, project];
    });
  }

  clear(): void {
    this._projects.set([]);
    this._error.set(null);
  }
}
