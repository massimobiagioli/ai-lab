import { Component, inject, input, output, signal } from '@angular/core';
import { StatusBadge, StatusTone } from '../../shared/status-badge/status-badge';
import { Button } from '../../shared/ui/button/button';
import { Icon } from '../../shared/ui/icon/icon';
import type { ConnectionState } from '../../core/ws/ws.service';
import { ApiClient } from '../../core/api/api-client';

@Component({
  selector: 'app-topbar',
  imports: [StatusBadge, Button, Icon],
  template: `
    <div class="topbar">
      <app-button class="topbar__menu" (pressed)="menu.emit()">
        <app-icon name="menu" />
        Menu
      </app-button>
      <div class="topbar__context">
        <span class="topbar__eyebrow">Workspace</span>
        <span class="topbar__lab">{{ labId() }}</span>
      </div>
      <div class="topbar__actions">
        @if (demo()) {
          <app-status-badge label="Demo mode" tone="warning" />
        } @else {
          <app-status-badge label="Realtime" [tone]="connectionTone()" />
        }
        <app-button (pressed)="toggleMute()" [variant]="muted() ? 'primary' : 'ghost'">
          {{ muted() ? 'Unmute' : 'Mute' }}
        </app-button>
        <app-button (pressed)="toggleFullscreen()">Fullscreen</app-button>
      </div>
    </div>
  `,
  styles: `
    .topbar {
      position: sticky;
      top: 0;
      z-index: 8;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-3);
      min-height: 4.25rem;
      padding: var(--ailab-space-3) clamp(1rem, 2.2vw, 2.25rem);
      border-bottom: var(--ailab-border);
      background: rgb(255 255 255 / 82%);
      backdrop-filter: blur(16px);
    }
    .topbar__context {
      display: flex;
      flex-direction: column;
      gap: 0.1rem;
      min-width: 0;
    }
    .topbar__eyebrow {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 650;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }
    .topbar__lab {
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-md);
      font-weight: 650;
    }
    .topbar__menu {
      display: none;
    }
    .topbar__actions {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-2);
    }
    @media (max-width: 900px) {
      .topbar__menu {
        display: inline-flex;
      }
      .topbar__actions app-button:not(:last-child) {
        display: none;
      }
    }
    @media (max-width: 520px) {
      .topbar__context {
        display: none;
      }
    }
  `,
})
export class Topbar {
  private readonly api = inject(ApiClient);
  readonly connection = input<ConnectionState>('offline');
  protected readonly demo = this.api.usingDemo;
  readonly labId = input('default');
  readonly menu = output<void>();
  protected readonly muted = signal(false);

  protected connectionTone(): StatusTone {
    switch (this.connection()) {
      case 'connected':
        return 'success';
      case 'reconnecting':
        return 'warning';
      default:
        return 'danger';
    }
  }

  protected toggleMute(): void {
    this.muted.update((value) => !value);
  }

  protected toggleFullscreen(): void {
    if (typeof document === 'undefined') {
      return;
    }
    if (document.fullscreenElement === null) {
      void document.documentElement.requestFullscreen?.().catch(() => undefined);
    } else {
      void document.exitFullscreen?.().catch(() => undefined);
    }
  }
}
