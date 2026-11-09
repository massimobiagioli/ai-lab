import { inject, Injectable, signal } from '@angular/core';
import { ApiClient } from '../../../core/api/api-client';
import { toProblem } from '../../../core/api/problem';
import type { LoadState } from '../../../core/state/store-utils';
import type { Problem, Sow } from '../../../domain/models';

/** Signal store for the SOW workspace. */
@Injectable({ providedIn: 'root' })
export class SowStore {
  private readonly api = inject(ApiClient);
  readonly state = signal<LoadState>('idle');
  readonly sow = signal<Sow | null>(null);
  readonly error = signal<Problem | null>(null);
  readonly demo = this.api.usingDemo;

  async load(projectId: string): Promise<void> {
    this.state.set('loading');
    this.error.set(null);
    try {
      this.sow.set(await this.api.getSow(projectId));
      this.state.set('ready');
    } catch (error) {
      this.error.set(toProblem(error));
      this.state.set('error');
    }
  }
}
