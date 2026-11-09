import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Button } from '../../shared/ui/button/button';
import { Card as UiCard } from '../../shared/ui/card/card';
import { Drawer } from '../../shared/ui/drawer/drawer';
import { EmptyState } from '../../shared/ui/empty-state/empty-state';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { Input } from '../../shared/ui/input/input';
import { Select } from '../../shared/ui/select/select';
import { StatusBadge } from '../../shared/status-badge/status-badge';
import { MemoryStore } from './store/memory.store';
import type { MemoryNote } from '../../domain/models';

@Component({
  selector: 'app-memory',
  imports: [PageHeader, Button, UiCard, Drawer, EmptyState, Spinner, Input, Select, StatusBadge],
  template: `
    <app-page-header title="Memory" subtitle="Scoped retrieval with citations">
      @if (store.demo()) {
        <app-status-badge label="Demo data" tone="warning" />
      }
    </app-page-header>

    <div class="filters">
      <app-input label="Search" [(value)]="query" placeholder="board conventions" />
      <app-select label="Scope" [options]="scopes" [(value)]="scope" />
      <app-button variant="primary" (pressed)="search()">Search</app-button>
    </div>

    @switch (store.state()) {
      @case ('loading') {
        <div class="center"><app-spinner label="Searching memory" /></div>
      }
      @case ('error') {
        <app-card title="Unable to search memory">
          <p>{{ store.error()?.detail ?? store.error()?.title ?? 'Unknown error' }}</p>
          <app-button (pressed)="search()">Retry</app-button>
        </app-card>
      }
      @case ('ready') {
        @if (results().length === 0) {
          <app-empty-state title="No notes" message="Try a different query or scope." />
        } @else {
          <div class="notes">
            @for (note of results(); track note.id) {
              <article
                class="note"
                data-testid="memory-note"
                tabindex="0"
                (click)="open(note)"
                (keydown.enter)="open(note)"
              >
                <header class="note__header">
                  <strong>{{ note.title }}</strong>
                  <app-status-badge [label]="note.scope" />
                  @if (isPinned(note.id)) {
                    <app-status-badge label="pinned" tone="success" />
                  }
                </header>
                <p class="note__snippet">{{ note.snippet }}</p>
                <p class="note__citations">Citations: {{ note.citations.join(', ') }}</p>
              </article>
            }
          </div>
        }
      }
      @default {
        <app-empty-state title="Memory" message="Press Search to query the memory." />
      }
    }

    <app-drawer label="Memory note" [open]="detailOpen()" (closed)="detailOpen.set(false)">
      @if (selected(); as note) {
        <h3>{{ note.title }}</h3>
        <p>
          Scope: <strong>{{ note.scope }}</strong>
        </p>
        <p>{{ note.snippet }}</p>
        <p>Citations: {{ note.citations.join(', ') }}</p>
        <div class="actions">
          <app-button (pressed)="togglePin(note.id)">{{
            isPinned(note.id) ? 'Unpin' : 'Pin'
          }}</app-button>
          <app-button variant="danger" (pressed)="remove(note.id)">Delete</app-button>
        </div>
      }
    </app-drawer>
  `,
  styles: `
    .filters {
      display: grid;
      grid-template-columns: minmax(180px, 1fr) 160px auto;
      gap: var(--ailab-space-2);
      align-items: end;
      margin-bottom: var(--ailab-space-4);
    }
    .notes {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      gap: var(--ailab-space-3);
    }
    .note {
      border: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface);
      padding: var(--ailab-space-3);
      cursor: pointer;
    }
    .note:focus-visible {
      outline: 2px solid var(--ailab-color-accent);
      outline-offset: 2px;
    }
    .note__header {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-2);
      flex-wrap: wrap;
    }
    .note__snippet {
      margin: var(--ailab-space-2) 0;
    }
    .note__citations {
      margin: 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .actions {
      display: flex;
      gap: var(--ailab-space-2);
      margin-top: var(--ailab-space-3);
    }
    .center {
      display: flex;
      justify-content: center;
      padding: var(--ailab-space-6);
    }
  `,
})
export class Memory implements OnInit {
  protected readonly store = inject(MemoryStore);
  protected readonly query = signal('');
  protected readonly scope = signal('all');
  protected readonly detailOpen = signal(false);
  protected readonly selected = signal<MemoryNote | null>(null);
  protected readonly scopes = [
    { value: 'all', label: 'All scopes' },
    { value: 'lab', label: 'Lab' },
    { value: 'project', label: 'Project' },
    { value: 'agent', label: 'Agent' },
  ] as const;
  protected readonly results = computed(() =>
    this.store.results().filter((note) => this.scope() === 'all' || note.scope === this.scope()),
  );

  ngOnInit(): void {
    void this.store.search('');
  }

  protected search(): void {
    void this.store.search(this.query());
  }

  protected open(note: MemoryNote): void {
    this.selected.set(note);
    this.detailOpen.set(true);
  }

  protected isPinned(id: string): boolean {
    return this.store.pinned().has(id);
  }

  protected togglePin(id: string): void {
    this.store.togglePin(id);
  }

  protected remove(id: string): void {
    this.store.remove(id);
    this.detailOpen.set(false);
  }
}
