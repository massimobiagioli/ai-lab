import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { input } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Button } from '../../shared/ui/button/button';
import { Card as UiCard } from '../../shared/ui/card/card';
import { EmptyState } from '../../shared/ui/empty-state/empty-state';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { StatusBadge, StatusTone } from '../../shared/status-badge/status-badge';
import { SowStore } from '../sow/store/sow.store';

@Component({
  selector: 'app-plan',
  imports: [PageHeader, Button, UiCard, EmptyState, Spinner, StatusBadge],
  template: `
    <app-page-header title="Plan" subtitle="WBS tree, dependencies and dry-run materialize" />

    @switch (store.state()) {
      @case ('loading') {
        <div class="center"><app-spinner label="Loading plan" /></div>
      }
      @case ('error') {
        <app-card title="Unable to load the plan">
          <p>{{ store.error()?.detail ?? store.error()?.title ?? 'Unknown error' }}</p>
          <app-button (pressed)="refresh()">Retry</app-button>
        </app-card>
      }
      @case ('ready') {
        @if (lanes().length === 0) {
          <app-empty-state title="No plan" message="Generate a SOW to build the plan." />
        } @else {
          <div class="lanes">
            @for (lane of lanes(); track lane.status) {
              <app-card [title]="lane.status">
                <ul class="tasks">
                  @for (task of lane.tasks; track task.id) {
                    <li class="task">
                      <span>{{ task.title }}</span>
                      <app-status-badge [label]="task.status" [tone]="tone(task.status)" />
                      @if (task.dependsOn.length > 0) {
                        <span class="deps">after {{ task.dependsOn.join(', ') }}</span>
                      }
                    </li>
                  }
                </ul>
              </app-card>
            }
          </div>
          <app-button variant="primary" (pressed)="dryRun()">Dry-run materialize</app-button>
          @if (preview()) {
            <p class="preview" role="status">
              {{ store.sow()?.tasks?.length ?? 0 }} tasks would be materialized (dry run).
            </p>
          }
        }
      }
    }
  `,
  styles: `
    .lanes {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: var(--ailab-space-3);
      margin-bottom: var(--ailab-space-3);
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
      flex-direction: column;
      gap: var(--ailab-space-1);
      border-left: 3px solid var(--ailab-color-accent);
      border-radius: 0 var(--ailab-radius-sm) var(--ailab-radius-sm) 0;
      background: var(--ailab-color-surface-raised);
      padding: var(--ailab-space-2) var(--ailab-space-3);
      padding-left: var(--ailab-space-2);
    }
    .deps {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .preview {
      color: var(--ailab-color-success);
      margin-top: var(--ailab-space-2);
    }
    .center {
      display: flex;
      justify-content: center;
      padding: var(--ailab-space-6);
    }
  `,
})
export class Plan implements OnInit {
  readonly projectId = input.required<string>();
  protected readonly store = inject(SowStore);
  protected readonly preview = signal(false);
  protected readonly lanes = computed(() => {
    const tasks = this.store.sow()?.tasks ?? [];
    return (['todo', 'doing', 'done'] as const)
      .map((status) => ({ status, tasks: tasks.filter((task) => task.status === status) }))
      .filter((lane) => lane.tasks.length > 0);
  });

  ngOnInit(): void {
    void this.store.load(this.projectId());
  }

  protected refresh(): void {
    void this.store.load(this.projectId());
  }

  protected dryRun(): void {
    this.preview.set(true);
  }

  protected tone(status: string): StatusTone {
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
