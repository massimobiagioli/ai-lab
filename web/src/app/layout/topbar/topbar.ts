import { Component, input, output, signal } from '@angular/core';
import { StatusBadge, StatusTone } from '../../shared/status-badge/status-badge';
import { Button } from '../../shared/ui/button/button';
import type { ConnectionState } from '../../core/ws/ws.service';

@Component({
  selector: 'app-topbar',
  imports: [StatusBadge, Button],
  template: `
    <div class="topbar">
      <app-button class="topbar__menu" (pressed)="menu.emit()">Menu</app-button>
      <div class="topbar__lab">
        <span class="topbar__label">Lab</span>
        <strong>{{ labId() }}</strong>
      </div>
      <div class="topbar__actions">
        <app-status-badge label="Realtime" [tone]="connectionTone()" />
        <app-button (pressed)="toggleMute()" [variant]="muted() ? 'primary' : 'ghost'">
          {{ muted() ? 'Unmute' : 'Mute' }}
        </app-button>
        <app-button (pressed)="toggleFullscreen()">Fullscreen</app-button>
      </div>
    </div>
  `,
  styles: `
    .topbar {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-3);
      padding: var(--ailab-space-3) var(--ailab-space-4);
      border-bottom: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-bg);
    }
    .topbar__lab {
      display: flex;
      align-items: baseline;
      gap: var(--ailab-space-2);
    }
    .topbar__label {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
      text-transform: uppercase;
      letter-spacing: 0.08em;
    }
    .topbar__menu {
      display: none;
    }
    @media (max-width: 767px) {
      .topbar__menu {
        display: inline-flex;
      }
    }
    .topbar__actions {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-2);
    }
  `,
})
export class Topbar {
  readonly connection = input<ConnectionState>('offline');
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
