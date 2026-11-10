import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';
import { Icon, type IconName } from '../../shared/ui/icon/icon';

interface NavItem {
  readonly label: string;
  readonly path: string;
  readonly icon: IconName;
}

interface NavGroup {
  readonly label: string;
  readonly items: readonly NavItem[];
}

const NAV: readonly NavGroup[] = [
  {
    label: 'Workspace',
    items: [
      { label: 'Dashboard', path: '/', icon: 'dashboard' },
      { label: 'Runs', path: '/runs', icon: 'runs' },
      { label: 'Virtual office', path: '/lab/default', icon: 'office' },
    ],
  },
  {
    label: 'Delivery',
    items: [
      { label: 'Board', path: '/projects/demo/board', icon: 'board' },
      { label: 'SOW', path: '/projects/demo/sow', icon: 'projects' },
    ],
  },
  {
    label: 'Manage',
    items: [
      { label: 'Agents', path: '/inventory/agents', icon: 'agents' },
      { label: 'Projects', path: '/inventory/projects', icon: 'projects' },
      { label: 'Providers', path: '/inventory/providers', icon: 'providers' },
      { label: 'Memory', path: '/memory', icon: 'memory' },
      { label: 'Settings', path: '/settings', icon: 'settings' },
    ],
  },
];

@Component({
  selector: 'app-sidebar',
  imports: [RouterLink, RouterLinkActive, Icon],
  template: `
    <nav class="sidebar" aria-label="Primary">
      <a class="sidebar__brand" routerLink="/" aria-label="AI-Lab home">
        <span class="sidebar__mark" aria-hidden="true">
          <app-icon name="dashboard" />
        </span>
        <span class="sidebar__brand-text">
          <strong>AI-Lab</strong>
          <small>Delivery studio</small>
        </span>
      </a>

      @for (group of nav; track group.label) {
        <section class="sidebar__group" [attr.aria-label]="group.label">
          <p class="sidebar__group-label">{{ group.label }}</p>
          <ul class="sidebar__list">
            @for (item of group.items; track item.path) {
              <li>
                <a
                  class="sidebar__link"
                  [routerLink]="item.path"
                  routerLinkActive="is-active"
                  [routerLinkActiveOptions]="{ exact: item.path === '/' }"
                >
                  <app-icon [name]="item.icon" />
                  <span class="sidebar__label">{{ item.label }}</span>
                </a>
              </li>
            }
          </ul>
        </section>
      }

      <div class="sidebar__footer">
        <span class="sidebar__pulse" aria-hidden="true"></span>
        <span>Local workspace</span>
      </div>
    </nav>
  `,
  styles: `
    .sidebar {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-6);
      width: 260px;
      padding: var(--ailab-space-5) var(--ailab-space-4);
      border-right: var(--ailab-border);
      background: var(--ailab-color-sidebar);
      min-height: 100dvh;
    }
    .sidebar__brand {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-3);
      padding: var(--ailab-space-1) var(--ailab-space-2);
      color: var(--ailab-color-text);
      text-decoration: none;
    }
    .sidebar__mark {
      display: grid;
      place-items: center;
      width: 2.5rem;
      height: 2.5rem;
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-gradient-accent);
      color: #ffffff;
      box-shadow: 0 10px 22px -14px var(--ailab-color-accent);
    }
    .sidebar__brand-text {
      display: flex;
      flex-direction: column;
      line-height: 1.2;
    }
    .sidebar__brand-text strong {
      font-size: 1.05rem;
      font-weight: 750;
      letter-spacing: -0.03em;
    }
    .sidebar__brand-text small {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 550;
      letter-spacing: 0.04em;
      text-transform: uppercase;
    }
    .sidebar__group {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-2);
    }
    .sidebar__group-label {
      margin: 0;
      padding: 0 var(--ailab-space-3);
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 700;
      letter-spacing: 0.09em;
      text-transform: uppercase;
    }
    .sidebar__list {
      list-style: none;
      margin: 0;
      padding: 0;
      display: flex;
      flex-direction: column;
      gap: 0.15rem;
    }
    .sidebar__link {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-3);
      min-height: 2.5rem;
      padding: 0.55rem var(--ailab-space-3);
      border: 1px solid transparent;
      border-radius: var(--ailab-radius-md);
      color: var(--ailab-color-text-soft);
      text-decoration: none;
      font-size: var(--ailab-font-size-md);
      font-weight: 550;
      transition:
        background var(--ailab-motion-fast) ease,
        color var(--ailab-motion-fast) ease,
        border-color var(--ailab-motion-fast) ease;
    }
    .sidebar__link:hover {
      background: var(--ailab-color-surface-raised);
      color: var(--ailab-color-text);
    }
    .sidebar__link.is-active {
      border-color: var(--ailab-color-accent-soft);
      background: var(--ailab-color-accent-soft);
      color: var(--ailab-color-accent);
      font-weight: 650;
    }
    .sidebar__link:focus-visible {
      outline: none;
      box-shadow: 0 0 0 3px var(--ailab-color-accent-soft);
    }
    .sidebar__label {
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .sidebar__footer {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-2);
      margin-top: auto;
      padding: var(--ailab-space-3);
      border: var(--ailab-border);
      border-radius: var(--ailab-radius-md);
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 600;
    }
    .sidebar__pulse {
      width: 0.45rem;
      height: 0.45rem;
      border-radius: 50%;
      background: var(--ailab-color-success);
      box-shadow: 0 0 0 4px var(--ailab-color-success-soft);
    }
  `,
})
export class Sidebar {
  protected readonly nav = NAV;
}
