import { Injectable, computed, signal } from '@angular/core';
import { ONBOARDING_STEP_COUNT } from '../models/onboarding.model';

export const STEP_LABELS = ['Welcome', 'Profile', 'AI Tools', 'Projects', 'Complete'] as const;

@Injectable({ providedIn: 'root' })
export class OnboardingWizardStore {
  private readonly _stepIndex = signal(0);

  readonly stepIndex = this._stepIndex.asReadonly();
  readonly stepCount = ONBOARDING_STEP_COUNT;
  readonly stepNumber = computed(() => this._stepIndex() + 1);
  readonly stepLabel = computed(() => STEP_LABELS[this._stepIndex()]);
  readonly progress = computed(() => ((this._stepIndex() + 1) / this.stepCount) * 100);

  next(): void {
    this._stepIndex.update(i => Math.min(i + 1, this.stepCount - 1));
  }

  back(): void {
    this._stepIndex.update(i => Math.max(i - 1, 0));
  }

  reset(): void {
    this._stepIndex.set(0);
  }
}
