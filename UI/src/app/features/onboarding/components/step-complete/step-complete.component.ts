import { Component, computed, inject, output, signal } from '@angular/core';
import type { OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { OnboardingService } from '../../services/onboarding.service';

@Component({
  selector: 'app-step-complete',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatIconModule],
  templateUrl: './step-complete.component.html',
  styleUrl: './step-complete.component.scss',
})
export class StepCompleteComponent implements OnInit {
  private readonly onboarding = inject(OnboardingService);

  readonly back = output<void>();
  readonly finished = output<void>();

  readonly finishing = signal(false);

  readonly user = this.onboarding.user;
  readonly enabledTools = computed(() => this.onboarding.aiTools().filter(t => t.isEnabled));
  readonly selectedProjects = computed(() => this.onboarding.projects().filter(p => p.isSelected));
  readonly enabledToolNames = computed(() => this.enabledTools().map(t => t.displayName).join(', '));

  async ngOnInit(): Promise<void> {
    await Promise.all([
      this.onboarding.loadUser(),
      this.onboarding.loadAiTools(),
      this.onboarding.loadProjects(),
    ]);
  }

  async finish(): Promise<void> {
    this.finishing.set(true);
    try {
      await this.onboarding.completeOnboarding();
      this.finished.emit();
    } finally {
      this.finishing.set(false);
    }
  }
}
