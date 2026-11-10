import { Component, inject, OnInit, signal } from '@angular/core';
import { Button } from '../../shared/ui/button/button';
import { RouterOutlet } from '@angular/router';
import { Sidebar } from '../sidebar/sidebar';
import { Topbar } from '../topbar/topbar';
import { Drawer } from '../../shared/ui/drawer/drawer';
import { WsService } from '../../core/ws/ws.service';
import { Telemetry } from '../../core/telemetry/telemetry.service';
import { ErrorState } from '../../core/errors/error-state';
import { DEMO_LAB_ID } from '../../core/demo/demo-data';

@Component({
  selector: 'app-shell',
  imports: [RouterOutlet, Sidebar, Topbar, Button, Drawer],
  template: `
    <a class="skip-link" href="#main">Skip to content</a>
    <div class="shell">
      <aside class="shell__sidebar"><app-sidebar /></aside>
      <div class="shell__content">
        <app-topbar [connection]="connection()" [labId]="labName()" (menu)="drawerOpen.set(true)" />
        @if (errorState.error(); as failure) {
          <div class="error-boundary" role="alert">
            <p>
              Something went wrong: {{ failure.message }}
              <span class="error-boundary__trace">trace {{ failure.traceId }}</span>
            </p>
            <div class="error-boundary__actions">
              <app-button variant="primary" (pressed)="retry()">Retry</app-button>
              <app-button (pressed)="dismissError()">Dismiss</app-button>
            </div>
          </div>
        }
        <main id="main" class="shell__main" tabindex="-1">
          <router-outlet />
        </main>
      </div>
    </div>
    <app-drawer label="Navigation" [open]="drawerOpen()" (closed)="drawerOpen.set(false)">
      <app-sidebar />
    </app-drawer>
    <div class="toast-host" role="status" aria-live="polite" aria-atomic="true">
      @if (toast() !== null) {
        <p class="toast">{{ toast() }}</p>
      }
    </div>
  `,
  styles: `
    .skip-link {
      position: absolute;
      left: -9999px;
      top: 0;
      padding: var(--ailab-space-2) var(--ailab-space-3);
      background: var(--ailab-color-accent);
      color: var(--ailab-color-bg);
      z-index: var(--ailab-z-toast);
    }
    .skip-link:focus {
      left: var(--ailab-space-2);
    }
    .shell {
      display: flex;
      min-height: 100dvh;
      background: transparent;
    }
    .shell__sidebar {
      position: sticky;
      top: 0;
      flex: 0 0 260px;
      height: 100dvh;
      overflow-y: auto;
      z-index: 10;
    }
    .shell__content {
      display: flex;
      flex-direction: column;
      flex: 1;
      min-width: 0;
    }
    .shell__main {
      width: 100%;
      max-width: 1440px;
      margin: 0 auto;
      padding: clamp(1rem, 2.2vw, 2.25rem);
      flex: 1;
    }
    .shell__main:focus {
      outline: none;
    }
    .toast-host {
      position: fixed;
      right: var(--ailab-space-4);
      bottom: var(--ailab-space-4);
      z-index: var(--ailab-z-toast);
    }
    .error-boundary {
      margin: var(--ailab-space-4) clamp(1rem, 2.2vw, 2.25rem) 0;
      border: 1px solid var(--ailab-color-danger-soft);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-danger-soft);
      color: var(--ailab-color-danger);
      padding: var(--ailab-space-3) var(--ailab-space-4);
    }
    .error-boundary__trace {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .error-boundary__actions {
      display: flex;
      gap: var(--ailab-space-2);
      margin-top: var(--ailab-space-2);
    }
    .toast {
      margin: 0;
      padding: var(--ailab-space-3) var(--ailab-space-4);
      border: 1px solid var(--ailab-color-border);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text);
      box-shadow: var(--ailab-shadow-raised);
    }
    @media (max-width: 900px) {
      .shell__sidebar {
        display: none;
      }
      .shell__main {
        padding: var(--ailab-space-4);
      }
    }
  `,
})
export class Shell implements OnInit {
  private readonly ws = inject(WsService);
  private readonly telemetry = inject(Telemetry);
  protected readonly errorState = inject(ErrorState);
  protected readonly labId = signal(DEMO_LAB_ID);
  protected readonly labName = signal('Demo Lab');
  protected readonly connection = this.ws.state;
  protected readonly toast = signal<string | null>(null);
  protected readonly drawerOpen = signal(false);

  ngOnInit(): void {
    this.ws.connect(this.labId());
    this.telemetry.capture('info', 'shell.ready');
  }

  protected retry(): void {
    this.errorState.clear();
    location.reload();
  }

  protected dismissError(): void {
    this.errorState.clear();
  }
}
