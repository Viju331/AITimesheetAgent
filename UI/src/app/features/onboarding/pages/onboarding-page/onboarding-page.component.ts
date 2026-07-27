import { Component, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Router } from '@angular/router';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { StepWelcomeComponent } from '../../components/step-welcome/step-welcome.component';
import { StepProfileComponent } from '../../components/step-profile/step-profile.component';
import { StepAiToolsComponent } from '../../components/step-ai-tools/step-ai-tools.component';
import { StepProjectsComponent } from '../../components/step-projects/step-projects.component';
import { StepCompleteComponent } from '../../components/step-complete/step-complete.component';
import { OnboardingWizardStore } from '../../store/onboarding-wizard.store';

@Component({
  selector: 'app-onboarding-page',
  standalone: true,
  imports: [
    CommonModule,
    MatProgressBarModule,
    StepWelcomeComponent,
    StepProfileComponent,
    StepAiToolsComponent,
    StepProjectsComponent,
    StepCompleteComponent,
  ],
  templateUrl: './onboarding-page.component.html',
  styleUrl: './onboarding-page.component.scss',
})
export class OnboardingPageComponent {
  private readonly router = inject(Router);
  readonly wizard = inject(OnboardingWizardStore);

  async onFinished(): Promise<void> {
    await this.router.navigate(['/dashboard']);
  }
}
