import { Component, inject, OnInit } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Button } from '../../shared/ui/button/button';
import { Card } from '../../shared/ui/card/card';
import { EmptyState } from '../../shared/ui/empty-state/empty-state';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { StatusBadge, StatusTone } from '../../shared/status-badge/status-badge';
import { RunsStore } from './runs.store';

@Component({
  selector: 'app-runs',
  imports: [PageHeader, Button, Card, EmptyState, Spinner, StatusBadge],
  template: `
    <app-page-header title="Runs" subtitle="Scheduler activity for the default lab">
      <app-button variant="primary" (pressed)="refresh()">Refresh</app-button>
    </app-page-header>

    @switch (store.state()) {
      @case ('loading') {
        <app-card><app-spinner label="Loading runs" /></app-card>
      }
      @case ('error') {
        <app-card title="Unable to load runs">
          <p>{{ store.error()?.detail ?? store.error()?.title ?? 'Unknown error' }}</p>
          <app-button (pressed)="refresh()">Retry</app-button>
        </app-card>
      }
      @case ('ready') {
        @if (store.isEmpty()) {
          <app-empty-state title="No runs yet" message="Queue a card to start the scheduler." />
        } @else {
          <app-card>
            <table class="runs">
              <caption class="runs__caption">
                Runs ordered by creation time
              </caption>
              <thead>
                <tr>
                  <th scope="col">Run</th>
                  <th scope="col">Card</th>
                  <th scope="col">State</th>
                  <th scope="col">Attempt</th>
                  <th scope="col">Started</th>
                  <th scope="col"><span class="runs__sr">Actions</span></th>
                </tr>
              </thead>
              <tbody>
                @for (run of store.runs(); track run.id) {
                  <tr>
                    <td>
                      <code>{{ run.id.slice(0, 8) }}</code>
                    </td>
                    <td>
                      <code>{{ run.card_id.slice(0, 8) }}</code>
                    </td>
                    <td><app-status-badge [label]="run.state" [tone]="tone(run.state)" /></td>
                    <td>{{ run.attempt }}</td>
                    <td>{{ run.started_at ?? run.created_at }}</td>
                    <td>
                      @if (run.state === 'running' || run.state === 'queued') {
                        <app-button variant="danger" (pressed)="store.cancel(run.id)">
                          Cancel
                        </app-button>
                      }
                    </td>
                  </tr>
                }
              </tbody>
            </table>
          </app-card>
        }
      }
      @default {
        <app-empty-state title="Runs" message="Press refresh to load the run list." />
      }
    }
  `,
  styles: `
    .runs {
      width: 100%;
      border-collapse: collapse;
      font-family: var(--ailab-font-sans);
      font-size: var(--ailab-font-size-md);
    }
    .runs__caption {
      text-align: left;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 650;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      padding-bottom: var(--ailab-space-2);
    }
    .runs th,
    .runs td {
      text-align: left;
      padding: 0.85rem 0.75rem;
      border-bottom: 1px solid var(--ailab-color-border);
    }
    .runs th {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 650;
      letter-spacing: 0.06em;
      text-transform: uppercase;
    }
    .runs code {
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
    }
    .runs__sr {
      position: absolute;
      width: 1px;
      height: 1px;
      overflow: hidden;
      clip: rect(0 0 0 0);
    }
  `,
})
export class Runs implements OnInit {
  protected readonly store = inject(RunsStore);

  ngOnInit(): void {
    void this.store.load();
  }

  protected refresh(): void {
    void this.store.load();
  }

  protected tone(state: string): StatusTone {
    switch (state) {
      case 'succeeded':
        return 'success';
      case 'running':
      case 'queued':
        return 'warning';
      case 'failed':
      case 'timed_out':
      case 'cancelled':
        return 'danger';
      default:
        return 'neutral';
    }
  }
}
