import { Component, inject, OnInit } from '@angular/core';
import { RouterLink } from '@angular/router';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Card } from '../../shared/ui/card/card';
import { Button } from '../../shared/ui/button/button';
import { RunsStore } from '../runs/runs.store';

@Component({
  selector: 'app-dashboard',
  imports: [PageHeader, Card, Button, RouterLink],
  template: `
    <app-page-header title="Dashboard" subtitle="Flow and run health for the default lab">
      <app-button variant="primary" (pressed)="refresh()">Refresh</app-button>
    </app-page-header>

    <div class="grid">
      <app-card title="Runs">
        <p class="metric">{{ store.counters().total }}</p>
        <p class="hint">Total tracked runs</p>
      </app-card>
      <app-card title="Running">
        <p class="metric">{{ store.counters().running }}</p>
        <p class="hint">Currently leased</p>
      </app-card>
      <app-card title="Succeeded">
        <p class="metric">{{ store.counters().succeeded }}</p>
        <p class="hint">Merged and done</p>
      </app-card>
      <app-card title="Failed">
        <p class="metric">{{ store.counters().failed }}</p>
        <p class="hint">Failed or timed out</p>
      </app-card>
    </div>

    <app-card title="Next actions">
      <p class="hint">
        Open the run list to inspect steps and cancel work, or jump to the board to queue a card.
      </p>
      <div class="actions">
        <a routerLink="/runs"><app-button variant="primary">Open runs</app-button></a>
        <a routerLink="/projects/demo/board"><app-button>Open board</app-button></a>
      </div>
    </app-card>
  `,
  styles: `
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      gap: var(--ailab-space-4);
      margin-bottom: var(--ailab-space-4);
    }
    .metric {
      margin: 0;
      font-size: var(--ailab-font-size-xl);
      color: var(--ailab-color-accent);
    }
    .hint {
      margin: var(--ailab-space-1) 0 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-md);
    }
    .actions {
      display: flex;
      gap: var(--ailab-space-2);
      margin-top: var(--ailab-space-3);
    }
    a {
      text-decoration: none;
    }
  `,
})
export class Dashboard implements OnInit {
  protected readonly store = inject(RunsStore);

  ngOnInit(): void {
    void this.store.load();
  }

  protected refresh(): void {
    void this.store.load();
  }
}
