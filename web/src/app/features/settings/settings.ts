import { Component, effect, inject, OnInit, signal } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Degraded } from '../../shared/ui/degraded/degraded';
import { Button } from '../../shared/ui/button/button';
import { Card as UiCard } from '../../shared/ui/card/card';
import { Dialog } from '../../shared/ui/dialog/dialog';
import { Input } from '../../shared/ui/input/input';
import { Select } from '../../shared/ui/select/select';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { StatusBadge } from '../../shared/status-badge/status-badge';
import { SettingsStore } from './store/settings.store';

@Component({
  selector: 'app-settings',
  imports: [PageHeader, Degraded, Button, UiCard, Dialog, Input, Select, Spinner, StatusBadge],
  template: `
    <app-degraded [active]="store.demo()" />
    <app-page-header title="Settings" subtitle="Lab, autonomy, budgets and office">
      @if (store.demo()) {
        <app-status-badge label="Demo data" tone="warning" />
      }
    </app-page-header>

    @switch (store.state()) {
      @case ('loading') {
        <div class="center"><app-spinner label="Loading settings" /></div>
      }
      @case ('error') {
        <app-card title="Unable to load settings">
          <p>{{ store.error()?.detail ?? store.error()?.title ?? 'Unknown error' }}</p>
          <app-button (pressed)="refresh()">Retry</app-button>
        </app-card>
      }
      @case ('ready') {
        <div class="grid">
          <app-card title="Lab">
            <app-input label="Lab name" [(value)]="labName" />
            <app-input label="Description" [(value)]="description" />
            <app-select label="Autonomy" [options]="autonomyOptions" [(value)]="autonomy" />
            <app-input
              label="Monthly budget (micro-USD)"
              [(value)]="budget"
              [error]="budgetError()"
            />
            <app-input label="Scheduler tick (ms)" [(value)]="tick" [error]="tickError()" />
            <app-input label="WIP limit" [(value)]="wip" />
            <app-input label="Scheduler cron" [(value)]="cron" />
            <app-select label="Office theme" [options]="themeOptions" [(value)]="theme" />
            <app-select label="Sound" [options]="soundOptions" [(value)]="sound" />
            <app-select
              label="Reduced motion"
              [options]="motionOptions"
              [(value)]="reducedMotion"
            />
            <div class="actions">
              <app-button variant="primary" (pressed)="save()">Save</app-button>
              <app-button (pressed)="exportSettings()">Export</app-button>
              @if (saved()) {
                <span class="saved" role="status"
                  >Saved in demo mode (audited in the real API).</span
                >
              }
            </div>
          </app-card>

          <app-card title="Exposure and providers">
            <p>Provider: <app-status-badge label="demo active" tone="success" /></p>
            <p>Exposure mode: <strong>loopback</strong> · the token is session-scoped only.</p>
            <p>Secrets are write-only; only the last 4 characters are shown.</p>
          </app-card>

          <app-card title="Audit">
            <ul>
              @for (entry of audit(); track $index) {
                <li>{{ entry }}</li>
              } @empty {
                <li>No changes yet.</li>
              }
            </ul>
          </app-card>

          <app-card title="Danger zone">
            <p>Archiving the lab hides it from the office and scheduler.</p>
            <div class="actions">
              <app-button variant="danger" (pressed)="archiveOpen.set(true)"
                >Archive lab</app-button
              >
              <app-button variant="danger" (pressed)="deleteOpen.set(true)">Delete lab</app-button>
            </div>
            @if (archived()) {
              <p class="saved" role="status">Lab archived in demo mode.</p>
            }
            @if (deleted()) {
              <p class="saved" role="status">Lab deleted in demo mode.</p>
            }
          </app-card>
        </div>
      }
    }

    <app-dialog
      label="Archive lab"
      title="Archive the lab?"
      [open]="archiveOpen()"
      (closed)="archiveOpen.set(false)"
      (confirmed)="archive()"
    >
      <p>Type ARCHIVE to confirm.</p>
      <app-input label="Confirmation" [(value)]="confirmText" />
    </app-dialog>

    <app-dialog
      label="Delete lab"
      title="Delete the lab?"
      [open]="deleteOpen()"
      (closed)="deleteOpen.set(false)"
      (confirmed)="deleteLab()"
    >
      <p>Type DELETE to confirm. This is irreversible.</p>
      <app-input label="Confirmation" [(value)]="confirmText" />
    </app-dialog>
  `,
  styles: `
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
      gap: var(--ailab-space-4);
    }
    .actions {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-2);
      margin-top: var(--ailab-space-3);
      flex-wrap: wrap;
    }
    .saved {
      color: var(--ailab-color-success);
      font-size: var(--ailab-font-size-sm);
    }
    .center {
      display: flex;
      justify-content: center;
      padding: var(--ailab-space-6);
    }
  `,
})
export class Settings implements OnInit {
  protected readonly store = inject(SettingsStore);
  protected readonly autonomy = signal('2');
  protected readonly budget = signal('50000000');
  protected readonly tick = signal('1000');
  protected readonly theme = signal('default');
  protected readonly labName = signal('AI-Lab Demo');
  protected readonly description = signal('Deterministic demo lab');
  protected readonly wip = signal('4');
  protected readonly cron = signal('*/5 * * * *');
  protected readonly sound = signal('off');
  protected readonly reducedMotion = signal('auto');
  protected readonly confirmText = signal('');
  protected readonly audit = signal<readonly string[]>([]);
  protected readonly deleted = signal(false);
  protected readonly deleteOpen = signal(false);
  protected readonly saved = signal(false);
  protected readonly archived = signal(false);
  protected readonly archiveOpen = signal(false);
  protected readonly autonomyOptions = [
    { value: '0', label: 'A0 — manual' },
    { value: '1', label: 'A1 — assisted' },
    { value: '2', label: 'A2 — supervised' },
    { value: '3', label: 'A3 — autonomous' },
  ] as const;
  protected readonly themeOptions = [
    { value: 'default', label: 'Default' },
    { value: 'high-contrast', label: 'High contrast' },
  ] as const;
  protected readonly soundOptions = [
    { value: 'off', label: 'Off' },
    { value: 'on', label: 'On' },
  ] as const;
  protected readonly motionOptions = [
    { value: 'auto', label: 'System' },
    { value: 'reduced', label: 'Reduced' },
  ] as const;
  protected readonly budgetError = signal<string | null>(null);
  protected readonly tickError = signal<string | null>(null);

  constructor() {
    effect(() => {
      const settings = this.store.settings();
      if (settings !== null) {
        this.autonomy.set(String(settings.autonomyLevel));
        this.budget.set(String(settings.monthlyBudgetMicrousd));
        this.tick.set(String(settings.schedulerTickMs));
        this.theme.set(settings.officeTheme);
      }
    });
  }

  ngOnInit(): void {
    void this.store.load();
  }

  protected refresh(): void {
    void this.store.load();
  }

  protected save(): void {
    this.saved.set(false);
    const budget = Number(this.budget());
    const tick = Number(this.tick());
    this.budgetError.set(
      Number.isFinite(budget) && budget >= 0 ? null : 'Budget must be a non-negative number',
    );
    this.tickError.set(
      Number.isFinite(tick) && tick >= 100 && tick <= 60_000
        ? null
        : 'Tick must be between 100 and 60000 ms',
    );
    if (this.budgetError() === null && this.tickError() === null) {
      this.saved.set(true);
      this.audit.update((entries) => [
        ...entries,
        `saved: lab=${this.labName()}, autonomy=${this.autonomy()}, budget=${this.budget()}, tick=${this.tick()}, wip=${this.wip()}, theme=${this.theme()}`,
      ]);
    }
  }

  protected exportSettings(): void {
    this.audit.update((entries) => [...entries, 'exported settings (demo)']);
  }

  protected archive(): void {
    if (this.confirmText() !== 'ARCHIVE') {
      return;
    }
    this.archiveOpen.set(false);
    this.archived.set(true);
    this.confirmText.set('');
    this.audit.update((entries) => [...entries, 'archived lab']);
  }

  protected deleteLab(): void {
    if (this.confirmText() !== 'DELETE') {
      return;
    }
    this.deleteOpen.set(false);
    this.deleted.set(true);
    this.confirmText.set('');
    this.audit.update((entries) => [...entries, 'deleted lab']);
  }
}
