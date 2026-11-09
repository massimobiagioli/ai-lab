import { computed, inject, Injectable, signal, type WritableSignal } from '@angular/core';
import { ApiClient } from '../../../core/api/api-client';
import { isDemoFallback } from '../../../core/demo/demo-data';
import { toProblem } from '../../../core/api/problem';
import { withOptimistic, type LoadState } from '../../../core/state/store-utils';
import type { Card, Problem } from '../../../domain/models';

/** Signal store for the Kanban board. */
@Injectable({ providedIn: 'root' })
export class BoardStore {
  private readonly api = inject(ApiClient);

  readonly state = signal<LoadState>('idle');
  readonly cards = signal<readonly Card[]>([]);
  readonly error = signal<Problem | null>(null);
  readonly demo = this.api.usingDemo;
  readonly columns = computed(() => {
    const groups = new Map<string, Card[]>();
    for (const card of this.cards()) {
      const list = groups.get(card.column) ?? [];
      list.push(card);
      groups.set(card.column, list);
    }
    return groups;
  });

  async load(projectId: string): Promise<void> {
    this.state.set('loading');
    this.error.set(null);
    try {
      this.cards.set(await this.api.getBoard(projectId));
      this.state.set('ready');
    } catch (error) {
      this.error.set(toProblem(error));
      this.state.set('error');
    }
  }

  async moveCard(cardId: string, to: string): Promise<void> {
    const snapshot = this.cards();
    const current = snapshot.find((card) => card.id === cardId);
    if (current === undefined) {
      return;
    }
    const optimistic = snapshot.map((card) =>
      card.id === cardId ? { ...card, column: to } : card,
    );
    const target: WritableSignal<readonly Card[]> = this.cards;
    try {
      await withOptimistic(
        target,
        optimistic,
        async () => {
          try {
            const updated = await this.api.transitionCard(cardId, to);
            this.cards.update((cards) =>
              cards.map((card) => (card.id === updated.id ? updated : card)),
            );
          } catch (error) {
            if (!isDemoFallback(error)) {
              throw error;
            }
          }
        },
        snapshot,
      );
    } catch (error) {
      this.error.set(toProblem(error));
    }
  }
}
