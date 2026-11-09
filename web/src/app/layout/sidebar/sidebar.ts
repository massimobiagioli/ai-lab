import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive } from '@angular/router';

interface NavItem {
  readonly label: string;
  readonly path: string;
  readonly icon: string;
}

const NAV: readonly NavItem[] = [
  { label: 'Dashboard', path: '/', icon: '▦' },
  { label: 'Runs', path: '/runs', icon: '▶' },
  { label: 'Office', path: '/lab/default', icon: '⌂' },
  { label: 'Board', path: '/projects/demo/board', icon: '▤' },
  { label: 'Agents', path: '/inventory/agents', icon: '☰' },
  { label: 'Projects', path: '/inventory/projects', icon: '◈' },
  { label: 'Providers', path: '/inventory/providers', icon: '⚙' },
  { label: 'SOW', path: '/projects/demo/sow', icon: '✎' },
  { label: 'Memory', path: '/memory', icon: '◍' },
  { label: 'Settings', path: '/settings', icon: '⚒' },
];

@Component({
  selector: 'app-sidebar',
  imports: [RouterLink, RouterLinkActive],
  template: `
    <nav class="sidebar" aria-label="Primary">
      <a class="sidebar__brand" routerLink="/" aria-label="AI-Lab home">AI·LAB</a>
      <ul class="sidebar__list">
        @for (item of nav; track item.path) {
          <li>
            <a
              class="sidebar__link"
              [routerLink]="item.path"
              routerLinkActive="is-active"
              [routerLinkActiveOptions]="{ exact: item.path === '/' }"
            >
              <span class="sidebar__icon" aria-hidden="true">{{ item.icon }}</span>
              <span class="sidebar__label">{{ item.label }}</span>
            </a>
          </li>
        }
      </ul>
    </nav>
  `,
  styles: `
    .sidebar {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-4);
      width: 240px;
      padding: var(--ailab-space-4);
      border-right: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface);
      min-height: 100dvh;
    }
    .sidebar__brand {
      color: var(--ailab-color-accent);
      font-size: var(--ailab-font-size-lg);
      letter-spacing: 0.2em;
      text-decoration: none;
    }
    .sidebar__list {
      list-style: none;
      margin: 0;
      padding: 0;
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-1);
    }
    .sidebar__link {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-2);
      padding: var(--ailab-space-2);
      color: var(--ailab-color-text);
      text-decoration: none;
      border: 2px solid transparent;
      font-size: var(--ailab-font-size-md);
    }
    .sidebar__link:hover {
      background: var(--ailab-color-surface-raised);
    }
    .sidebar__link.is-active {
      border-color: var(--ailab-color-accent);
      color: var(--ailab-color-accent);
    }
    .sidebar__link:focus-visible {
      outline: 2px solid var(--ailab-color-accent);
      outline-offset: 2px;
    }
    .sidebar__icon {
      width: 1.2em;
      text-align: center;
    }
    @media (max-width: 767px) {
      .sidebar {
        width: auto;
        min-height: auto;
        flex-direction: row;
        overflow-x: auto;
        border-right: 0;
        border-bottom: 2px solid var(--ailab-color-muted);
      }
      .sidebar__list {
        flex-direction: row;
      }
      .sidebar__label {
        display: none;
      }
    }
  `,
})
export class Sidebar {
  protected readonly nav = NAV;
}
