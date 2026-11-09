import { Component, inject, OnInit, signal } from '@angular/core';
import { Button } from '../../shared/ui/button/button';
import { RouterOutlet } from '@angular/router';
import { Sidebar } from '../sidebar/sidebar';
import { Topbar } from '../topbar/topbar';
import { WsService } from '../../core/ws/ws.service';
import { Telemetry } from '../../core/telemetry/telemetry.service';
import { ErrorState } from '../../core/errors/error-state';

@Component({
  selector: 'app-shell',
  imports: [RouterOutlet, Sidebar, Topbar, Button],
  template: `
    <a class="skip-link" href="#main">Skip to content</a>
    <div class="shell">
      <app-sidebar />
      <div class="shell__content">
        <app-topbar [connection]="connection()" [labId]="labId()" />
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
    }
    .shell__content {
      display: flex;
      flex-direction: column;
      flex: 1;
      min-width: 0;
    }
    .shell__main {
      padding: var(--ailab-space-4);
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
      border-bottom: 2px solid var(--ailab-color-danger);
      background: var(--ailab-color-surface);
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
      padding: var(--ailab-space-2) var(--ailab-space-3);
      border: 2px solid var(--ailab-color-accent);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-accent);
    }
    @media (max-width: 767px) {
      .shell {
        flex-direction: column;
      }
    }
  `,
})
export class Shell implements OnInit {
  private readonly ws = inject(WsService);
  private readonly telemetry = inject(Telemetry);
  protected readonly errorState = inject(ErrorState);
  protected readonly labId = signal('default');
  protected readonly connection = this.ws.state;
  protected readonly toast = signal<string | null>(null);

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
