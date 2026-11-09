import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { input } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Card as UiCard } from '../../shared/ui/card/card';
import { Button } from '../../shared/ui/button/button';
import { PixelTable } from '../../shared/ui/pixel-table/pixel-table';
import { Drawer } from '../../shared/ui/drawer/drawer';
import { Dialog } from '../../shared/ui/dialog/dialog';
import { EmptyState } from '../../shared/ui/empty-state/empty-state';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { Input } from '../../shared/ui/input/input';
import { StatusBadge } from '../../shared/status-badge/status-badge';
import { InventoryStore } from './store/inventory.store';
import type { Agent, Project, Provider } from '../../domain/models';

export type InventorySection = 'agents' | 'projects' | 'providers';

interface Column {
  readonly key: string;
  readonly label: string;
}

@Component({
  selector: 'app-inventory',
  imports: [
    PageHeader,
    Button,
    PixelTable,
    Drawer,
    Dialog,
    UiCard,
    EmptyState,
    Spinner,
    Input,
    StatusBadge,
  ],
  template: `
    <app-page-header [title]="title()" [subtitle]="subtitle()">
      @if (store.demo()) {
        <app-status-badge label="Demo data" tone="warning" />
      }
      <app-button variant="primary" (pressed)="openNew()">New</app-button>
    </app-page-header>

    @switch (store.state()) {
      @case ('loading') {
        <div class="center"><app-spinner label="Loading inventory" /></div>
      }
      @case ('error') {
        <app-card title="Unable to load">
          <p>{{ store.error()?.detail ?? store.error()?.title ?? 'Unknown error' }}</p>
          <app-button (pressed)="refresh()">Retry</app-button>
        </app-card>
      }
      @case ('ready') {
        @if (rows().length === 0) {
          <app-empty-state title="Nothing here yet" message="Create the first record with New." />
        } @else {
          <app-pixel-table
            [caption]="title()"
            [columns]="columns()"
            [rows]="rows()"
            (rowSelected)="selectRow($event)"
          />
        }
      }
      @default {
        <app-empty-state title="Inventory" message="Press New or refresh to load." />
      }
    }

    <app-drawer label="Detail" [open]="detailOpen()" (closed)="detailOpen.set(false)">
      @if (selected(); as item) {
        <h3>{{ item.name }}</h3>
        @for (entry of entries(item); track entry[0]) {
          <p>
            {{ entry[0] }}: <strong>{{ entry[1] }}</strong>
          </p>
        }
      }
    </app-drawer>

    <app-dialog
      label="Create"
      [title]="'New ' + sectionLabel()"
      [open]="newOpen()"
      (closed)="closeNew()"
      (confirmed)="create()"
    >
      <app-input label="Name" [(value)]="name" [error]="formError()" />
    </app-dialog>
  `,
  styles: `
    .center {
      display: flex;
      justify-content: center;
      padding: var(--ailab-space-6);
    }
  `,
})
export class Inventory implements OnInit {
  readonly section = input.required<InventorySection>();
  protected readonly store = inject(InventoryStore);
  protected readonly detailOpen = signal(false);
  protected readonly newOpen = signal(false);
  protected readonly name = signal('');
  protected readonly formError = signal<string | null>(null);
  protected readonly selected = signal<Agent | Project | Provider | null>(null);

  protected readonly title = computed(() => this.sectionLabel());
  protected readonly subtitle = computed(() => `Manage ${this.sectionLabel().toLowerCase()}.`);
  protected readonly sectionLabel = computed(() => {
    switch (this.section()) {
      case 'agents':
        return 'Agents';
      case 'projects':
        return 'Projects';
      default:
        return 'Providers';
    }
  });

  protected readonly columns = computed<readonly Column[]>(() => {
    switch (this.section()) {
      case 'agents':
        return [
          { key: 'name', label: 'Name' },
          { key: 'role', label: 'Role' },
          { key: 'status', label: 'Status' },
          { key: 'model', label: 'Model' },
        ];
      case 'projects':
        return [
          { key: 'name', label: 'Name' },
          { key: 'repo', label: 'Repository' },
          { key: 'branch', label: 'Branch' },
          { key: 'autonomyLevel', label: 'Autonomy' },
        ];
      default:
        return [
          { key: 'name', label: 'Name' },
          { key: 'kind', label: 'Kind' },
          { key: 'status', label: 'Status' },
          { key: 'last4', label: 'Key' },
        ];
    }
  });

  protected readonly rows = computed<readonly Readonly<Record<string, string>>[]>(() => {
    switch (this.section()) {
      case 'agents':
        return this.store.agents().map((agent) => ({
          name: agent.name,
          role: agent.role,
          status: agent.status,
          model: agent.model,
        }));
      case 'projects':
        return this.store.projects().map((project) => ({
          name: project.name,
          repo: project.repo,
          branch: project.branch,
          autonomyLevel: String(project.autonomyLevel),
        }));
      default:
        return this.store.providers().map((provider) => ({
          name: provider.name,
          kind: provider.kind,
          status: provider.status,
          last4: provider.last4,
        }));
    }
  });

  ngOnInit(): void {
    void this.store.load();
  }

  protected refresh(): void {
    void this.store.load();
  }

  protected selectRow(index: number): void {
    const items =
      this.section() === 'agents'
        ? this.store.agents()
        : this.section() === 'projects'
          ? this.store.projects()
          : this.store.providers();
    this.selected.set(items[index] ?? null);
    this.detailOpen.set(true);
  }

  protected entries(item: Agent | Project | Provider): readonly (readonly [string, string])[] {
    if ('role' in item) {
      return [
        ['Role', item.role],
        ['Status', item.status],
        ['Model', item.model],
        ['Concurrency', String(item.maxConcurrent)],
      ];
    }
    if ('repo' in item) {
      return [
        ['Repository', item.repo],
        ['Branch', item.branch],
        ['Autonomy', String(item.autonomyLevel)],
      ];
    }
    return [
      ['Kind', item.kind],
      ['Status', item.status],
      ['Key', `••••${item.last4}`],
    ];
  }

  protected openNew(): void {
    this.name.set('');
    this.formError.set(null);
    this.newOpen.set(true);
  }

  protected closeNew(): void {
    this.newOpen.set(false);
  }

  protected create(): void {
    const name = this.name().trim();
    if (name.length < 2) {
      this.formError.set('Name must be at least 2 characters');
      return;
    }
    const id = `local-${crypto.randomUUID()}`;
    switch (this.section()) {
      case 'agents':
        this.store.addAgent({
          id,
          name,
          role: 'dev',
          status: 'idle',
          autonomyLevel: 2,
          model: 'deepseek-flash',
          maxConcurrent: 1,
        });
        break;
      case 'projects':
        this.store.addProject({
          id,
          labId: 'default',
          name,
          repo: `ai-lab/${name.toLowerCase().replace(/\s+/g, '-')}`,
          branch: 'main',
          autonomyLevel: 2,
        });
        break;
      default:
        this.store.addProvider({
          id,
          name,
          kind: 'deepseek',
          status: 'active',
          last4: '0000',
        });
    }
    this.newOpen.set(false);
  }
}
