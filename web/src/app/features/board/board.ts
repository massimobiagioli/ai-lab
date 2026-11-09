import { Component, computed, effect, inject, OnInit, signal } from '@angular/core';
import { input } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Button } from '../../shared/ui/button/button';
import { Card as UiCard } from '../../shared/ui/card/card';
import { Drawer } from '../../shared/ui/drawer/drawer';
import { EmptyState } from '../../shared/ui/empty-state/empty-state';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { StatusBadge, StatusTone } from '../../shared/status-badge/status-badge';
import { Select } from '../../shared/ui/select/select';
import { BoardStore } from './store/board.store';
import { WsService } from '../../core/ws/ws.service';
import type { Card } from '../../domain/models';

const COLUMNS = [
  { id: 'TODO', label: 'TODO' },
  { id: 'READY', label: 'READY' },
  { id: 'DOING', label: 'DOING' },
  { id: 'REVIEW', label: 'REVIEW' },
  { id: 'DONE', label: 'DONE' },
  { id: 'ICEBOX', label: 'ICEBOX' },
] as const;

@Component({
  selector: 'app-board',
  imports: [PageHeader, Button, UiCard, Drawer, EmptyState, Spinner, StatusBadge, Select],
  template: `
    <app-page-header title="Kanban board" subtitle="Six columns, keyboard moves and violations">
      @if (store.demo()) {
        <app-status-badge label="Demo data" tone="warning" />
      }
      <app-button variant="primary" (pressed)="refresh()">Refresh</app-button>
    </app-page-header>

    <div class="filters">
      <app-select label="Assignee" [options]="assigneeOptions()" [(value)]="assignee" />
    </div>

    @switch (store.state()) {
      @case ('loading') {
        <div class="center"><app-spinner label="Loading board" /></div>
      }
      @case ('error') {
        <app-card title="Unable to load the board">
          <p>{{ store.error()?.detail ?? store.error()?.title ?? 'Unknown error' }}</p>
          <app-button (pressed)="refresh()">Retry</app-button>
        </app-card>
      }
      @case ('ready') {
        @if (store.cards().length === 0) {
          <app-empty-state title="No cards" message="Create a card to populate the board." />
        } @else {
          <div class="board">
            @for (column of columns; track column.id) {
              <section
                class="column"
                [attr.data-testid]="'column-' + column.id"
                (dragover)="allowDrop($event)"
                (drop)="dropOn(column.id)"
              >
                <h2 class="column__title">{{ column.label }} ({{ cardsFor(column.id).length }})</h2>
                @for (card of cardsFor(column.id); track card.id) {
                  <article
                    class="card"
                    [attr.data-testid]="'card-' + card.key"
                    draggable="true"
                    [class.is-moving]="moveMode() === card.id"
                    (dragstart)="dragCard.set(card.id)"
                    (dragend)="dragCard.set(null)"
                    (click)="openCard(card)"
                    (keydown)="onCardKey($event, card)"
                    tabindex="0"
                  >
                    <header class="card__header">
                      <strong>{{ card.key }}</strong>
                      <app-status-badge
                        [label]="card.priority"
                        [tone]="priorityTone(card.priority)"
                      />
                    </header>
                    <p class="card__title">{{ card.title }}</p>
                    <p class="card__meta">
                      {{ card.assigneeId ?? 'unassigned' }}
                      @if (card.blockedReason !== null) {
                        · <span class="card__blocked">{{ card.blockedReason }}</span>
                      }
                    </p>
                  </article>
                }
              </section>
            }
          </div>
        }
      }
      @default {
        <app-empty-state title="Board" message="Press refresh to load the board." />
      }
    }

    <p class="announce" aria-live="polite">{{ announcement() }}</p>
    @if (store.lastViolation(); as violation) {
      <div class="violation" role="alert">
        <span>{{ violation }}</span>
        <app-button (pressed)="store.clearViolation()">Dismiss</app-button>
      </div>
    }

    <app-drawer label="Card" [open]="detailOpen()" (closed)="detailOpen.set(false)">
      @if (selectedCard(); as card) {
        <h3>{{ card.key }} · {{ card.title }}</h3>
        <p>
          Column: <strong>{{ card.column }}</strong>
        </p>
        <p>
          Priority: <strong>{{ card.priority }}</strong>
        </p>
        <p>Assignee: {{ card.assigneeId ?? 'unassigned' }}</p>
        <p>Estimate: {{ card.estimate ?? '—' }}</p>
        @if (card.blockedReason !== null) {
          <p class="card__blocked">{{ card.blockedReason }}</p>
        }
        @if (card.prUrl !== null) {
          <p>
            <a [href]="card.prUrl" rel="noopener">{{ card.prUrl }}</a>
          </p>
        }
      }
    </app-drawer>
  `,
  styles: `
    .filters {
      margin-bottom: var(--ailab-space-3);
      max-width: 240px;
    }
    .board {
      display: grid;
      grid-template-columns: repeat(6, minmax(180px, 1fr));
      gap: var(--ailab-space-2);
      overflow-x: auto;
      align-items: start;
    }
    .column {
      background: var(--ailab-color-surface-sunken);
      border: 2px solid var(--ailab-color-surface-raised);
      padding: var(--ailab-space-2);
      min-height: 240px;
    }
    .column__title {
      margin: 0 0 var(--ailab-space-2);
      font-size: var(--ailab-font-size-sm);
      letter-spacing: 0.08em;
      color: var(--ailab-color-muted);
    }
    .card {
      background: var(--ailab-color-surface);
      border: 2px solid var(--ailab-color-muted);
      padding: var(--ailab-space-2);
      margin-bottom: var(--ailab-space-2);
      cursor: grab;
    }
    .card:focus-visible {
      outline: 2px solid var(--ailab-color-accent);
      outline-offset: 2px;
    }
    .card.is-moving {
      border-color: var(--ailab-color-accent);
    }
    .card__header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-1);
    }
    .card__title {
      margin: var(--ailab-space-1) 0;
      font-size: var(--ailab-font-size-md);
    }
    .card__meta {
      margin: 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .card__blocked {
      color: var(--ailab-color-danger);
    }
    .announce {
      color: var(--ailab-color-accent);
      font-size: var(--ailab-font-size-sm);
      min-height: 1.2em;
    }
    .violation {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-2);
      border: 2px solid var(--ailab-color-danger);
      color: var(--ailab-color-danger);
      padding: var(--ailab-space-2);
      margin-top: var(--ailab-space-2);
    }
    .center {
      display: flex;
      justify-content: center;
      padding: var(--ailab-space-6);
    }
    @media (max-width: 900px) {
      .board {
        grid-template-columns: repeat(6, 200px);
      }
    }
  `,
})
export class Board implements OnInit {
  readonly projectId = input.required<string>();
  readonly card = input<string | null>(null);
  readonly assignee = signal<string>('all');
  protected readonly store = inject(BoardStore);
  private readonly ws = inject(WsService);
  protected readonly columns = COLUMNS;
  protected readonly detailOpen = signal(false);
  protected readonly selectedCard = signal<Card | null>(null);
  protected readonly moveMode = signal<string | null>(null);
  protected readonly moveTarget = signal<string>('TODO');
  protected readonly dragCard = signal<string | null>(null);
  protected readonly announcement = signal('');
  protected readonly assigneeOptions = computed(() => {
    const ids = new Set(this.store.cards().map((card) => card.assigneeId ?? 'unassigned'));
    return [
      { value: 'all', label: 'All assignees' },
      ...[...ids].sort().map((id) => ({ value: id, label: id })),
    ];
  });

  constructor() {
    effect(() => {
      const event = this.ws.lastEvent();
      if (event?.event.startsWith('kanban.card') || event?.event.startsWith('delivery.')) {
        void this.store.load(this.projectId());
      }
    });
  }

  ngOnInit(): void {
    void this.store.load(this.projectId());
    const cardKey = this.card();
    if (cardKey !== null) {
      const found = this.store.cards().find((item) => item.key === cardKey);
      if (found) {
        this.openCard(found);
      }
    }
  }

  protected refresh(): void {
    void this.store.load(this.projectId());
  }

  protected cardsFor(column: string): readonly Card[] {
    const assignee = this.assignee();
    return (this.store.columns().get(column) ?? []).filter(
      (card) => assignee === 'all' || (card.assigneeId ?? 'unassigned') === assignee,
    );
  }

  protected openCard(card: Card): void {
    this.selectedCard.set(card);
    this.detailOpen.set(true);
  }

  protected allowDrop(event: DragEvent): void {
    event.preventDefault();
  }

  protected dropOn(column: string): void {
    const id = this.dragCard();
    if (id !== null) {
      void this.store.moveCard(id, column);
      this.announcement.set(`Moved ${id} to ${column}`);
      this.dragCard.set(null);
    }
  }

  protected onCardKey(event: KeyboardEvent, card: Card): void {
    if (event.key.toLowerCase() === 'm') {
      event.preventDefault();
      this.moveMode.set(card.id);
      this.moveTarget.set(card.column);
      this.announcement.set(`Moving ${card.key}. Use arrows and Enter, Escape to cancel.`);
      return;
    }
    if (this.moveMode() !== card.id) {
      return;
    }
    const index = COLUMNS.findIndex((column) => column.id === this.moveTarget());
    if (event.key === 'ArrowRight' && index < COLUMNS.length - 1) {
      event.preventDefault();
      this.moveTarget.set(COLUMNS[index + 1]?.id ?? this.moveTarget());
      this.announcement.set(`Target ${this.moveTarget()}`);
    } else if (event.key === 'ArrowLeft' && index > 0) {
      event.preventDefault();
      this.moveTarget.set(COLUMNS[index - 1]?.id ?? this.moveTarget());
      this.announcement.set(`Target ${this.moveTarget()}`);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      void this.store.moveCard(card.id, this.moveTarget());
      this.announcement.set(`Moved ${card.key} to ${this.moveTarget()}`);
      this.moveMode.set(null);
    } else if (event.key === 'Escape') {
      this.moveMode.set(null);
      this.announcement.set('Move cancelled');
    }
  }

  protected priorityTone(priority: string): StatusTone {
    switch (priority) {
      case 'must':
        return 'danger';
      case 'should':
        return 'warning';
      default:
        return 'neutral';
    }
  }
}
