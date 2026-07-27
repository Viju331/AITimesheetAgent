import { Component, computed, inject, output, signal } from '@angular/core';
import type { OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatTableModule } from '@angular/material/table';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { OnboardingService } from '../../services/onboarding.service';
import type { ProjectDto } from '../../models/onboarding.model';

@Component({
  selector: 'app-step-projects',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatIconModule,
    MatCheckboxModule,
    MatTableModule,
    MatProgressSpinnerModule,
  ],
  templateUrl: './step-projects.component.html',
  styleUrl: './step-projects.component.scss',
})
export class StepProjectsComponent implements OnInit {
  private readonly onboarding = inject(OnboardingService);

  readonly back = output<void>();
  readonly next = output<void>();

  readonly rootFolder = this.onboarding.rootFolder;
  readonly projects = this.onboarding.projects;
  readonly isScanning = this.onboarding.isScanning;

  readonly searchTerm = signal('');
  readonly columns = ['select', 'name', 'techStack', 'path', 'lastScanned'];

  readonly filteredProjects = computed(() => {
    const term = this.searchTerm().trim().toLowerCase();
    const all = this.projects();
    if (!term) return all;
    return all.filter(
      p => p.name.toLowerCase().includes(term) || p.path.toLowerCase().includes(term),
    );
  });

  readonly allSelected = computed(() => {
    const all = this.projects();
    return all.length > 0 && all.every(p => p.isSelected);
  });

  readonly selectedCount = computed(() => this.projects().filter(p => p.isSelected).length);

  async ngOnInit(): Promise<void> {
    await this.onboarding.loadRootFolder();
    await this.onboarding.loadProjects();
  }

  async pickRootFolder(): Promise<void> {
    const path = await this.onboarding.pickFolder();
    if (path) {
      await this.onboarding.saveRootFolder(path);
      await this.scan();
    }
  }

  async scan(): Promise<void> {
    const root = this.rootFolder();
    if (!root) return;
    await this.onboarding.discoverProjects([root]);
  }

  async toggleProject(project: ProjectDto): Promise<void> {
    await this.onboarding.setProjectSelection(project.id, !project.isSelected);
  }

  async toggleAll(): Promise<void> {
    await this.onboarding.setAllProjectsSelection(!this.allSelected());
  }
}
