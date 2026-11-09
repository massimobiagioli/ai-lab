import { inject, Injectable, signal } from '@angular/core';
import { ApiClient } from '../../../core/api/api-client';
import { toProblem } from '../../../core/api/problem';
import type { LoadState } from '../../../core/state/store-utils';
import type { LabSettings, Problem } from '../../../domain/models';

/** Signal store for lab settings. */
@Injectable({ providedIn: 'root' })
export class SettingsStore {
  private readonly api = inject(ApiClient);
  readonly state = signal<LoadState>('idle');
  readonly settings = signal<LabSettings | null>(null);
  readonly error = signal<Problem | null>(null);
  readonly demo = this.api.usingDemo;

  async load(labId = 'default'): Promise<void> {
    this.state.set('loading');
    this.error.set(null);
    try {
      this.settings.set(await this.api.getSettings(labId));
      this.state.set('ready');
    } catch (error) {
      this.error.set(toProblem(error));
      this.state.set('error');
    }
  }
}
