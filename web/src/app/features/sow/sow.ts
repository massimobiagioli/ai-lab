import { Component, computed, effect, inject, OnInit, signal } from '@angular/core';
import { input } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Button } from '../../shared/ui/button/button';
import { Card as UiCard } from '../../shared/ui/card/card';
import { Dialog } from '../../shared/ui/dialog/dialog';
import { EmptyState } from '../../shared/ui/empty-state/empty-state';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { StatusBadge, StatusTone } from '../../shared/status-badge/status-badge';
import { Input } from '../../shared/ui/input/input';
import { SowStore } from './store/sow.store';

@Component({
  selector: 'app-sow',
  imports: [PageHeader, Button, UiCard, Dialog, EmptyState, Spinner, StatusBadge, Input],
  template: `
    <app-page-header title="SOW workspace" subtitle="Sections, delivery graph and materialize">
      @if (store.demo()) {
        <app-status-badge label="Demo data" tone="warning" />
      }
      <app-button variant="primary" (pressed)="refresh()">Refresh</app-button>
    </app-page-header>

    @switch (store.state()) {
      @case ('loading') {
        <div class="center"><app-spinner label="Loading SOW" /></div>
      }
      @case ('error') {
        <app-card title="Unable to load the SOW">
          <p>{{ store.error()?.detail ?? store.error()?.title ?? 'Unknown error' }}</p>
          <app-button (pressed)="refresh()">Retry</app-button>
        </app-card>
      }
      @case ('ready') {
        @if (store.sow(); as sow) {
          <div class="sow">
            <app-card title="Versions">
              <p>
                Version <strong>v{{ sow.version }}</strong> ·
                <app-status-badge [label]="sow.status" [tone]="tone(sow.status)" />
              </p>
              <app-button variant="primary" (pressed)="confirmOpen.set(true)"
                >Materialize</app-button
              >
              @if (materialized()) {
                <p class="sow__done" role="status">
                  Materialized in demo mode (real API in WP-13B T07).
                </p>
              }
            </app-card>

            <app-card title="Sections">
              @for (section of sow.sections; track section.id) {
                <article class="section">
                  <h3>{{ section.title }}</h3>
                  <p>{{ section.body }}</p>
                </article>
              }
            </app-card>

            <app-card title="Delivery graph">
              <ul class="tasks">
                @for (task of sow.tasks; track task.id) {
                  <li class="task">
                    <span class="task__title">{{ task.title }}</span>
                    <app-status-badge [label]="task.status" [tone]="taskTone(task.status)" />
                    @if (task.dependsOn.length > 0) {
                      <span class="task__deps">depends on {{ task.dependsOn.join(', ') }}</span>
                    }
                  </li>
                }
              </ul>
            </app-card>
          </div>
        } @else {
          <app-empty-state title="No SOW" message="Generate a SOW to populate this workspace." />
        }
      }
      @default {
        <app-empty-state title="SOW" message="Press refresh to load the SOW." />
      }
    }

    <app-dialog
      label="Approve"
      title="Approve this SOW?"
      [open]="approveOpen()"
      (closed)="approveOpen.set(false)"
      (confirmed)="decide('approved')"
    >
      <p>The SOW becomes approved and can be materialized.</p>
    </app-dialog>

    <app-dialog
      label="Reject"
      title="Reject this SOW"
      [open]="rejectOpen()"
      (closed)="rejectOpen.set(false)"
      (confirmed)="decide('rejected')"
    >
      <app-input label="Reason" [(value)]="rejectReason" />
    </app-dialog>

    <app-dialog
      label="Traceability"
      title="Traceability"
      [open]="traceOpen()"
      (closed)="traceOpen.set(false)"
      (confirmed)="traceOpen.set(false)"
    >
      <ul>
        @for (task of store.sow()?.tasks ?? []; track task.id) {
          <li>{{ task.id }} → {{ task.title }} ({{ task.status }})</li>
        }
      </ul>
    </app-dialog>

    <app-dialog
      label="Dry run"
      title="Materialize preview"
      [open]="dryRunOpen()"
      (closed)="dryRunOpen.set(false)"
      (confirmed)="dryRunOpen.set(false)"
    >
      <p>
        {{ store.sow()?.tasks?.length ?? 0 }} tasks and
        {{ store.sow()?.sections?.length ?? 0 }} sections would be materialized.
      </p>
    </app-dialog>

    <app-dialog
      label="Materialize"
      title="Materialize the SOW?"
      [open]="confirmOpen()"
      (closed)="confirmOpen.set(false)"
      (confirmed)="materialize()"
    >
      <p>This creates the board cards and tasks. In demo mode nothing is persisted.</p>
    </app-dialog>
  `,
  styles: `
    .sow {
      display: grid;
      gap: var(--ailab-space-4);
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    }
    .section h3 {
      margin: 0 0 var(--ailab-space-1);
    }
    .section p {
      margin: 0 0 var(--ailab-space-3);
    }
    .tasks {
      list-style: none;
      margin: 0;
      padding: 0;
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-2);
    }
    .task {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-2);
      flex-wrap: wrap;
      border-left: 2px solid var(--ailab-color-accent);
      padding-left: var(--ailab-space-2);
    }
    .task__deps {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .sow__done {
      color: var(--ailab-color-success);
      font-size: var(--ailab-font-size-sm);
    }
    .actions {
      display: flex;
      flex-wrap: wrap;
      gap: var(--ailab-space-1);
      margin-top: var(--ailab-space-2);
    }
    .center {
      display: flex;
      justify-content: center;
      padding: var(--ailab-space-6);
    }
  `,
})
export class Sow implements OnInit {
  readonly projectId = input.required<string>();
  protected readonly store = inject(SowStore);
  protected readonly confirmOpen = signal(false);
  protected readonly approveOpen = signal(false);
  protected readonly rejectOpen = signal(false);
  protected readonly traceOpen = signal(false);
  protected readonly dryRunOpen = signal(false);
  protected readonly rejectReason = signal('');
  protected readonly status = signal('');
  protected readonly history = signal<readonly string[]>([]);
  protected readonly materialized = signal(false);
  protected readonly title = computed(() => this.store.sow()?.id ?? 'SOW');

  constructor() {
    effect(() => {
      const sow = this.store.sow();
      if (sow !== null && this.status() === '') {
        this.status.set(sow.status);
      }
    });
  }

  ngOnInit(): void {
    void this.store.load(this.projectId());
  }

  protected refresh(): void {
    void this.store.load(this.projectId());
  }

  protected decide(next: 'approved' | 'rejected'): void {
    this.status.set(next);
    this.history.update((entries) => [
      ...entries,
      `${next}${next === 'rejected' && this.rejectReason().trim() !== '' ? `: ${this.rejectReason()}` : ''}`,
    ]);
    this.approveOpen.set(false);
    this.rejectOpen.set(false);
    this.rejectReason.set('');
  }

  protected materialize(): void {
    this.confirmOpen.set(false);
    this.materialized.set(true);
    this.history.update((entries) => [...entries, 'materialized (demo)']);
  }

  protected tone(status: string): StatusTone {
    switch (status) {
      case 'approved':
        return 'success';
      case 'rejected':
        return 'danger';
      default:
        return 'warning';
    }
  }

  protected taskTone(status: string): StatusTone {
    switch (status) {
      case 'done':
        return 'success';
      case 'doing':
        return 'warning';
      default:
        return 'neutral';
    }
  }
}
