import { inject } from '@angular/core';
import { Router, type CanActivateFn } from '@angular/router';
import { OnboardingService } from '../services/onboarding.service';

/** Guards app routes that require onboarding to be complete; redirects to /onboarding otherwise. */
export const requireOnboardingGuard: CanActivateFn = async () => {
  const onboarding = inject(OnboardingService);
  const router = inject(Router);

  const state = await onboarding.loadState();
  return state.isComplete ? true : router.createUrlTree(['/onboarding']);
};

/** Guards the /onboarding route itself; redirects to /dashboard if already complete. */
export const skipOnboardingIfCompleteGuard: CanActivateFn = async () => {
  const onboarding = inject(OnboardingService);
  const router = inject(Router);

  const state = await onboarding.loadState();
  return state.isComplete ? router.createUrlTree(['/dashboard']) : true;
};
