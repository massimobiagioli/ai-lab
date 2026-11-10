import { Component, computed, inject, OnInit, signal } from '@angular/core';
import { input } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Degraded } from '../../shared/ui/degraded/degraded';
import { Card as UiCard } from '../../shared/ui/card/card';
import { Button } from '../../shared/ui/button/button';
import { PixelTable } from '../../shared/ui/pixel-table/pixel-table';
import { Drawer } from '../../shared/ui/drawer/drawer';
import { Dialog } from '../../shared/ui/dialog/dialog';
import { EmptyState } from '../../shared/ui/empty-state/empty-state';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { Input } from '../../shared/ui/input/input';
import { Select } from '../../shared/ui/select/select';
import { StatusBadge } from '../../shared/status-badge/status-badge';
import { Tabs } from '../../shared/ui/tabs/tabs';
import { PixelCheckbox } from '../../shared/ui/pixel-checkbox/pixel-checkbox';
import { DEMO_MEMORY } from '../../core/demo/demo-data';
import { InventoryStore } from './store/inventory.store';
import type { Agent, Project, Provider } from '../../domain/models';
import { createId } from '../../core/ids';

export type InventorySection = 'agents' | 'projects' | 'providers';

interface Column {
  readonly key: string;
  readonly label: string;
}

@Component({
  selector: 'app-inventory',
  imports: [
    PageHeader,
    Degraded,
    Tabs,
    PixelCheckbox,
    Select,
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
    <app-degraded [active]="store.demo()" />
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

    <app-drawer
      label="Agent Inspector"
      [open]="inspectorOpen()"
      (closed)="inspectorOpen.set(false)"
    >
      @if (inspectorAgent(); as agent) {
        <h3>{{ agent.name }}</h3>
        <app-tabs label="Inspector" [tabs]="inspectorTabs" [(active)]="activeTab" />
        @switch (activeTab()) {
          @case ('cv') {
            <p>
              Role: <strong>{{ agent.role }}</strong>
            </p>
            <p>
              Autonomy: <strong>{{ agent.autonomyLevel ?? 'inherited' }}</strong>
            </p>
            <p>
              Model: <strong>{{ agent.model }}</strong>
            </p>
            <p>
              Concurrency: <strong>{{ agent.maxConcurrent }}</strong>
            </p>
          }
          @case ('memory') {
            <ul>
              @for (note of memoryNotes; track note.id) {
                <li>{{ note.title }} — {{ note.snippet }}</li>
              }
            </ul>
          }
          @case ('controls') {
            <div class="actions">
              @if (agent.status !== 'working') {
                <app-button variant="primary" (pressed)="setAgentStatus(agent.id, 'working')"
                  >Resume</app-button
                >
              } @else {
                <app-button (pressed)="setAgentStatus(agent.id, 'paused')">Pause</app-button>
              }
              <app-button variant="danger" (pressed)="retire(agent.id)">Retire</app-button>
            </div>
          }
          @default {
            <p>
              Status: <strong>{{ agent.status }}</strong>
            </p>
            <p>
              Model: <strong>{{ agent.model }}</strong>
            </p>
            <p>Live steps: idle (no active run).</p>
          }
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
      @switch (section()) {
        @case ('agents') {
          <app-select label="Role" [options]="roleOptions" [(value)]="role" />
          <app-select label="Character" [options]="characterOptions" [(value)]="character" />
          <app-input label="Name" [(value)]="name" [error]="formError()" />
          <app-select label="Model" [options]="modelOptions" [(value)]="model" />
          <app-select label="Autonomy" [options]="autonomyChoices" [(value)]="autonomy" />
        }
        @case ('projects') {
          <app-input label="Name" [(value)]="name" [error]="formError()" />
          <app-input label="Repository" [(value)]="repo" placeholder="owner/repo" />
          <app-input label="Branch" [(value)]="branch" />
          <app-select label="Autonomy" [options]="autonomyChoices" [(value)]="autonomy" />
        }
        @default {
          <app-input label="Name" [(value)]="name" [error]="formError()" />
          <app-input
            label="API key"
            type="password"
            [(value)]="providerKey"
            [error]="formError()"
          />
          <app-pixel-checkbox label="Verify before saving" [(checked)]="verify" />
        }
      }
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
  protected readonly inspectorAgent = computed<Agent | null>(() => {
    const item = this.selected();
    return item !== null && 'role' in item ? item : null;
  });
  protected readonly inspectorOpen = signal(false);
  protected readonly activeTab = signal('live');
  protected readonly role = signal('dev');
  protected readonly character = signal('scrappy_dev');
  protected readonly model = signal('deepseek-flash');
  protected readonly autonomy = signal('2');
  protected readonly repo = signal('');
  protected readonly branch = signal('main');
  protected readonly providerKey = signal('');
  protected readonly verify = signal(true);
  protected readonly inspectorTabs = [
    { id: 'live', label: 'Live' },
    { id: 'cv', label: 'CV' },
    { id: 'memory', label: 'Memory' },
    { id: 'controls', label: 'Controls' },
  ] as const;
  protected readonly autonomyChoices = [
    { value: '0', label: 'A0' },
    { value: '1', label: 'A1' },
    { value: '2', label: 'A2' },
    { value: '3', label: 'A3' },
  ] as const;
  protected readonly roleOptions = [
    { value: 'tech_lead', label: 'Tech Lead' },
    { value: 'analyst', label: 'Analyst' },
    { value: 'dev', label: 'Dev' },
    { value: 'qa', label: 'QA' },
  ] as const;
  protected readonly characterOptions = [
    { value: 'veteran_lead', label: 'The Veteran Lead' },
    { value: 'systems_analyst', label: 'The Systems Analyst' },
    { value: 'scrappy_dev', label: 'The Scrappy Dev' },
    { value: 'staff_architect', label: 'The Staff Architect' },
    { value: 'precise_consultant', label: 'The Precise Consultant' },
    { value: 'confident_muse', label: 'The Confident Muse' },
    { value: 'steady_operator', label: 'The Steady Operator' },
    { value: 'eager_intern', label: 'The Eager Intern' },
    { value: 'pixel_artisan', label: 'The Pixel Artisan' },
    { value: 'quiet_professor', label: 'The Quiet Professor' },
  ] as const;
  protected readonly modelOptions = [
    { value: 'deepseek-flash', label: 'deepseek-flash' },
    { value: 'deepseek-v4', label: 'deepseek-v4' },
    { value: 'deepseek-v4-pro', label: 'deepseek-v4-pro' },
  ] as const;
  protected readonly memoryNotes = DEMO_MEMORY;

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
    const item = items[index] ?? null;
    this.selected.set(item);
    if (this.section() === 'agents') {
      this.inspectorOpen.set(true);
      this.activeTab.set('live');
    } else {
      this.detailOpen.set(true);
    }
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

  protected setAgentStatus(id: string, status: Agent['status']): void {
    this.store.setAgentStatus(id, status);
  }

  protected retire(id: string): void {
    this.store.removeAgent(id);
    this.inspectorOpen.set(false);
  }

  protected create(): void {
    const name = this.name().trim();
    if (name.length < 2) {
      this.formError.set('Name must be at least 2 characters');
      return;
    }
    if (this.section() === 'providers' && this.providerKey().length < 8) {
      this.formError.set('API key must be at least 8 characters');
      return;
    }
    const id = `local-${createId()}`;
    switch (this.section()) {
      case 'agents':
        this.store.addAgent({
          id,
          name,
          role: this.role() as Agent['role'],
          status: 'idle',
          autonomyLevel: Number(this.autonomy()),
          model: this.model(),
          maxConcurrent: 1,
        });
        break;
      case 'projects':
        this.store.addProject({
          id,
          labId: 'default',
          name,
          repo:
            this.repo().trim() === ''
              ? `ai-lab/${name.toLowerCase().replace(/\s+/g, '-')}`
              : this.repo(),
          branch: this.branch() || 'main',
          autonomyLevel: Number(this.autonomy()),
        });
        break;
      default:
        this.store.addProvider({
          id,
          name,
          kind: 'deepseek',
          status: 'active',
          last4: this.providerKey().slice(-4),
        });
    }
    this.newOpen.set(false);
  }
}
