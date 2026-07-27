import { Component, inject, signal, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ReactiveFormsModule, FormBuilder, FormGroup } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatButtonModule } from '@angular/material/button';
import { MatDividerModule } from '@angular/material/divider';
import { MatIconModule } from '@angular/material/icon';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { SettingsService } from '../../services/settings.service';
import { SettingsStore } from '../../../../store/settings.store';
import { invoke } from '@tauri-apps/api/core';

@Component({
  selector: 'app-settings-page',
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatSelectModule,
    MatButtonModule,
    MatDividerModule,
    MatIconModule,
    MatSnackBarModule,
  ],
  templateUrl: './settings-page.component.html',
  styleUrl: './settings-page.component.scss',
})
export class SettingsPageComponent implements OnInit {
  private readonly fb       = inject(FormBuilder);
  private readonly service  = inject(SettingsService);
  private readonly store    = inject(SettingsStore);
  private readonly snackbar = inject(MatSnackBar);

  readonly saving = signal(false);

  readonly form: FormGroup = this.fb.group({
    theme:          [this.store.state().theme],
    claudePath:     [this.store.state().claudePath ?? ''],
    cursorPath:     [this.store.state().cursorPath ?? ''],
    codexPath:      [this.store.state().codexPath  ?? ''],
    projectRoot:    [this.store.state().projectRoot ?? ''],
    openAiKey:      [''],
    azureOpenAiKey: [''],
  });

  ngOnInit(): void {
    const s = this.store.state();
    this.form.patchValue({
      theme:       s.theme,
      claudePath:  s.claudePath  ?? '',
      cursorPath:  s.cursorPath  ?? '',
      codexPath:   s.codexPath   ?? '',
      projectRoot: s.projectRoot ?? '',
    });
  }

  async pickFolder(controlName: string): Promise<void> {
    const path = await invoke<string | null>('pick_folder');
    if (path) this.form.get(controlName)?.setValue(path);
  }

  async save(): Promise<void> {
    if (this.form.invalid) return;
    this.saving.set(true);
    try {
      const v = this.form.value as Record<string, string>;
      await this.service.saveTheme(v['theme']);
      await this.service.saveAiPaths({
        claudePath: v['claudePath'] || undefined,
        cursorPath: v['cursorPath'] || undefined,
        codexPath:  v['codexPath']  || undefined,
      });
      if (v['openAiKey'])      await this.service.saveApiKeys({ openAiKey:      v['openAiKey'] });
      if (v['azureOpenAiKey']) await this.service.saveApiKeys({ azureOpenAiKey: v['azureOpenAiKey'] });
      this.snackbar.open('Settings saved', 'OK', { duration: 2500 });
    } finally {
      this.saving.set(false);
    }
  }
}
