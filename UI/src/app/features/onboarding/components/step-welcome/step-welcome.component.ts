import { Component, output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';

@Component({
  selector: 'app-step-welcome',
  standalone: true,
  imports: [CommonModule, MatButtonModule, MatIconModule],
  templateUrl: './step-welcome.component.html',
  styleUrl: './step-welcome.component.scss',
})
export class StepWelcomeComponent {
  readonly next = output<void>();

  readonly benefits = [
    { icon: 'smart_toy', text: 'Automatically reads your AI coding sessions (Claude Code, Cursor, Codex)' },
    { icon: 'merge_type', text: 'Combines AI session activity with your git history' },
    { icon: 'edit_note', text: 'Learns your writing style and drafts timesheets that sound like you' },
    { icon: 'verified', text: 'You stay in control — review and edit before anything is submitted' },
  ];
}
