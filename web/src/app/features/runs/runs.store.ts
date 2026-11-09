import { computed, inject, Injectable, signal, type WritableSignal } from '@angular/core';
import { ApiClient } from '../../core/api/api-client';
import { toProblem } from '../../core/api/problem';
import { withOptimistic, type LoadState } from '../../core/state/store-utils';
import type { Problem, RunView } from '../../domain/models';

export type StoreState = LoadState;

/** Signal store for the runs list. */
@Injectable({ providedIn: 'root' })
export class RunsStore {
  private readonly api = inject(ApiClient);

  readonly state = signal<StoreState>('idle');
  readonly runs = signal<readonly RunView[]>([]);
  readonly error = signal<Problem | null>(null);
  readonly isEmpty = computed(() => this.state() === 'ready' && this.runs().length === 0);
  readonly counters = computed(() => {
    const runs = this.runs();
    return {
      total: runs.length,
      running: runs.filter((run) => run.state === 'running').length,
      succeeded: runs.filter((run) => run.state === 'succeeded').length,
      failed: runs.filter((run) => run.state === 'failed' || run.state === 'timed_out').length,
    };
  });

  async load(labId = 'default'): Promise<void> {
    this.state.set('loading');
    this.error.set(null);
    try {
      const page = await this.api.listRuns(labId);
      this.runs.set(page.data);
      this.state.set('ready');
    } catch (error) {
      this.error.set(toProblem(error));
      this.state.set('error');
    }
  }

  async cancel(runId: string): Promise<void> {
    const snapshot = this.runs();
    const optimistic = snapshot.map((run) =>
      run.id === runId ? { ...run, state: 'cancelled' as const } : run,
    );
    const target: WritableSignal<readonly RunView[]> = this.runs;
    try {
      await withOptimistic(
        target,
        optimistic,
        async () => {
          const updated = await this.api.cancelRun(runId);
          this.runs.update((runs) => runs.map((run) => (run.id === updated.id ? updated : run)));
        },
        snapshot,
      );
    } catch (error) {
      this.error.set(toProblem(error));
    }
  }
}
