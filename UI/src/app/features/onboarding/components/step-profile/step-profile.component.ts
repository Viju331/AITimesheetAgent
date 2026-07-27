import { Component, inject, output, signal } from '@angular/core';
import type { OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, ReactiveFormsModule, Validators } from '@angular/forms';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { OnboardingService } from '../../services/onboarding.service';

@Component({
  selector: 'app-step-profile',
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatIconModule,
  ],
  templateUrl: './step-profile.component.html',
  styleUrl: './step-profile.component.scss',
})
export class StepProfileComponent implements OnInit {
  private readonly fb = inject(FormBuilder);
  private readonly onboarding = inject(OnboardingService);

  readonly back = output<void>();
  readonly next = output<void>();

  readonly saving = signal(false);

  readonly form = this.fb.group({
    name: ['', Validators.required],
    email: [''],
  });

  async ngOnInit(): Promise<void> {
    await this.onboarding.loadUser();
    const user = this.onboarding.user();
    if (user) {
      this.form.patchValue({ name: user.name, email: user.email ?? '' });
    }
  }

  async save(): Promise<void> {
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }

    this.saving.set(true);
    try {
      const v = this.form.value;
      await this.onboarding.saveUser(v.name ?? '', v.email || null);
      this.next.emit();
    } finally {
      this.saving.set(false);
    }
  }

  skip(): void {
    this.next.emit();
  }
}
