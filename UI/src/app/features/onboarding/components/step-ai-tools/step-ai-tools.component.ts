import { Component, inject, output, signal } from '@angular/core';
import type { OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { OnboardingService } from '../../services/onboarding.service';
import type { AiToolDto } from '../../models/onboarding.model';

@Component({
  selector: 'app-step-ai-tools',
  standalone: true,
  imports: [
    CommonModule,
    MatCheckboxModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatIconModule,
  ],
  templateUrl: './step-ai-tools.component.html',
  styleUrl: './step-ai-tools.component.scss',
})
export class StepAiToolsComponent implements OnInit {
  private readonly onboarding = inject(OnboardingService);

  readonly back = output<void>();
  readonly next = output<void>();

  readonly tools = this.onboarding.aiTools;
  readonly loading = signal(false);

  async ngOnInit(): Promise<void> {
    this.loading.set(true);
    try {
      await this.onboarding.loadAiTools();
    } finally {
      this.loading.set(false);
    }
  }

  async toggleTool(tool: AiToolDto): Promise<void> {
    await this.onboarding.updateAiTool(tool.id, !tool.isEnabled, tool.sessionFolder);
  }

  async pickFolder(tool: AiToolDto): Promise<void> {
    const path = await this.onboarding.pickFolder();
    if (path) {
      await this.onboarding.updateAiTool(tool.id, true, path);
    }
  }

  async onFolderInput(tool: AiToolDto, value: string): Promise<void> {
    await this.onboarding.updateAiTool(tool.id, tool.isEnabled, value || null);
  }
}
