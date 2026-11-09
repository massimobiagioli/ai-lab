import { Component, signal } from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Card } from '../../shared/ui/card/card';
import { Button } from '../../shared/ui/button/button';
import { EmptyState } from '../../shared/ui/empty-state/empty-state';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { StatusBadge } from '../../shared/status-badge/status-badge';
import { Input } from '../../shared/ui/input/input';
import { Select } from '../../shared/ui/select/select';
import { Tabs } from '../../shared/ui/tabs/tabs';
import { Dialog } from '../../shared/ui/dialog/dialog';
import { Toast } from '../../shared/ui/toast/toast';
import { Breadcrumbs } from '../../shared/ui/breadcrumbs/breadcrumbs';
import { Pagination } from '../../shared/ui/pagination/pagination';
import { Skeleton } from '../../shared/ui/skeleton/skeleton';
import { Progress } from '../../shared/ui/progress/progress';
import { Avatar } from '../../shared/ui/avatar/avatar';
import { Icon } from '../../shared/ui/icon/icon';
import { Menu } from '../../shared/ui/menu/menu';
import { Divider } from '../../shared/ui/divider/divider';
import { Kbd } from '../../shared/ui/kbd/kbd';
import { PixelText } from '../../shared/ui/pixel-text/pixel-text';
import { PixelPanel } from '../../shared/ui/pixel-panel/pixel-panel';
import { PixelTable } from '../../shared/ui/pixel-table/pixel-table';
import { PixelIconButton } from '../../shared/ui/pixel-icon-button/pixel-icon-button';
import { PixelTextarea } from '../../shared/ui/pixel-textarea/pixel-textarea';
import { PixelCheckbox } from '../../shared/ui/pixel-checkbox/pixel-checkbox';

@Component({
  selector: 'app-design-gallery',
  imports: [
    PageHeader,
    Card,
    Button,
    EmptyState,
    Spinner,
    StatusBadge,
    Input,
    Select,
    Tabs,
    Dialog,
    Toast,
    Breadcrumbs,
    Pagination,
    Skeleton,
    Progress,
    Avatar,
    Icon,
    Menu,
    Divider,
    Kbd,
    PixelText,
    PixelPanel,
    PixelTable,
    PixelIconButton,
    PixelTextarea,
    PixelCheckbox,
  ],
  template: `
    <app-page-header title="Design system" subtitle="Pixel primitives and their states" />
    <app-breadcrumbs [items]="crumbs" />

    <div class="grid">
      <app-card title="Buttons">
        <div class="row">
          <app-button variant="primary">Primary</app-button>
          <app-button>Ghost</app-button>
          <app-button variant="danger">Danger</app-button>
          <app-button [disabled]="true">Disabled</app-button>
        </div>
      </app-card>

      <app-card title="Badges & toast">
        <div class="row">
          <app-status-badge label="READY" tone="success" />
          <app-status-badge label="BLOCKED" tone="danger" />
          <app-status-badge label="DOING" tone="warning" />
          <app-status-badge label="TODO" />
        </div>
        <app-toast class="mt" message="Saved" tone="success" />
      </app-card>

      <app-card title="Form">
        <app-input label="Search" [(value)]="query" placeholder="card key" />
        <app-select label="Scope" [options]="scopes" [(value)]="scope" />
      </app-card>

      <app-card title="Tabs">
        <app-tabs label="Sections" [tabs]="tabs" [(active)]="tab" />
        <p>Active: {{ tab() }}</p>
      </app-card>

      <app-pixel-panel title="Mandated primitives" [raised]="true">
        <p class="row">
          <app-pixel-icon-button icon="settings" label="Settings" />
          <app-pixel-checkbox label="Done" [(checked)]="done" />
        </p>
        <app-pixel-textarea label="Notes" [(value)]="notes" />
        <app-pixel-table caption="Sample table" [columns]="tableColumns" [rows]="tableRows" />
      </app-pixel-panel>

      <app-card title="Data">
        <app-progress [value]="60" label="Run progress" />
        <app-skeleton width="80%" />
        <app-skeleton width="50%" />
        <app-divider />
        <app-pagination label="1 of 3" [hasNext]="true" />
        <app-menu label="Actions" [items]="menuItems" />
      </app-card>

      <app-card title="Identity & icons">
        <p class="row"><app-pixel-text text="AI LAB" /></p>
        <div class="row">
          <app-avatar name="Ada Lovelace" />
          <app-icon name="dashboard" />
          <app-icon name="runs" />
          <app-icon name="board" />
          <app-icon name="settings" />
        </div>
        <p>Press <app-kbd>Esc</app-kbd> to close.</p>
      </app-card>

      <app-card title="Feedback">
        <app-spinner label="Loading" />
        <app-empty-state title="No cards" message="Create a card to start." />
      </app-card>

      <app-card title="Dialog">
        <app-button variant="primary" (pressed)="dialogOpen.set(true)">Open dialog</app-button>
        <app-dialog
          label="Confirm"
          title="Confirm action"
          [open]="dialogOpen()"
          (closed)="dialogOpen.set(false)"
          (confirmed)="dialogOpen.set(false)"
        >
          <p>This is a pixel dialog with a focus trap.</p>
        </app-dialog>
      </app-card>
    </div>
  `,
  styles: `
    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
      gap: var(--ailab-space-4);
    }
    .row {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: var(--ailab-space-2);
    }
    .mt {
      margin-top: var(--ailab-space-3);
    }
  `,
})
export class DesignGallery {
  protected readonly query = signal('');
  protected readonly notes = signal('');
  protected readonly done = signal(true);
  protected readonly tableColumns = [
    { key: 'key', label: 'Key' },
    { key: 'state', label: 'State' },
  ] as const;
  protected readonly tableRows = [
    { key: 'AIL-1', state: 'DONE' },
    { key: 'AIL-2', state: 'DOING' },
  ] as const;
  protected readonly scope = signal('all');
  protected readonly tab = signal('overview');
  protected readonly dialogOpen = signal(false);
  protected readonly tabs = [
    { id: 'overview', label: 'Overview' },
    { id: 'steps', label: 'Steps' },
    { id: 'logs', label: 'Logs' },
  ] as const;
  protected readonly scopes = [
    { value: 'all', label: 'All' },
    { value: 'lab', label: 'Lab' },
  ] as const;
  protected readonly menuItems = [
    { id: 'open', label: 'Open' },
    { id: 'cancel', label: 'Cancel' },
  ] as const;
  protected readonly crumbs = [
    { label: 'Dashboard', path: '/' },
    { label: 'Design', path: '/design' },
  ] as const;
}
