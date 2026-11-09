import { inject, Injectable, signal } from '@angular/core';
import { ApiClient } from '../../../core/api/api-client';
import { toProblem } from '../../../core/api/problem';
import type { LoadState } from '../../../core/state/store-utils';
import type { MemoryNote, Problem } from '../../../domain/models';

/** Signal store for the memory explorer. */
@Injectable({ providedIn: 'root' })
export class MemoryStore {
  private readonly api = inject(ApiClient);
  readonly state = signal<LoadState>('idle');
  readonly results = signal<readonly MemoryNote[]>([]);
  readonly error = signal<Problem | null>(null);
  readonly demo = this.api.usingDemo;

  async search(query: string): Promise<void> {
    this.state.set('loading');
    this.error.set(null);
    try {
      this.results.set(await this.api.searchMemory(query));
      this.state.set('ready');
    } catch (error) {
      this.error.set(toProblem(error));
      this.state.set('error');
    }
  }
}
