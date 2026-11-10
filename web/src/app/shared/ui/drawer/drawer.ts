import { Component, input, output } from '@angular/core';
import { Button } from '../button/button';
import { FocusTrap } from '../../directives/focus-trap';

@Component({
  selector: 'app-drawer',
  host: { '(document:keydown.escape)': 'onEscape()' },
  imports: [Button, FocusTrap],
  template: `
    @if (open()) {
      <div
        class="drawer"
        [class.drawer--right]="side() === 'right'"
        role="dialog"
        aria-modal="true"
        [attr.aria-label]="label()"
        appFocusTrap
      >
        <header class="drawer__header">
          <h2 class="drawer__title">{{ label() }}</h2>
          <app-button (pressed)="closed.emit()">Close</app-button>
        </header>
        <div class="drawer__body"><ng-content /></div>
      </div>
      <button
        class="drawer__scrim"
        type="button"
        aria-label="Close drawer"
        (click)="closed.emit()"
      ></button>
    }
  `,
  styles: `
    .drawer {
      position: fixed;
      inset: 0 auto 0 0;
      width: min(380px, 92vw);
      background: var(--ailab-color-surface);
      border-right: var(--ailab-border);
      box-shadow: var(--ailab-shadow-raised);
      padding: var(--ailab-space-6);
      z-index: var(--ailab-z-drawer);
      overflow-y: auto;
    }
    .drawer--right {
      inset: 0 0 0 auto;
      border-right: 0;
      border-left: var(--ailab-border);
    }
    .drawer__header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-2);
      margin-bottom: var(--ailab-space-4);
    }
    .drawer__title {
      margin: 0;
      font-size: 1.1rem;
      font-weight: 700;
      letter-spacing: -0.02em;
    }
    .drawer__scrim {
      position: fixed;
      inset: 0;
      background: var(--ailab-color-overlay);
      border: 0;
      z-index: calc(var(--ailab-z-drawer) - 1);
      cursor: pointer;
    }
  `,
})
export class Drawer {
  readonly label = input.required<string>();
  readonly open = input.required<boolean>();
  readonly side = input<'left' | 'right'>('left');
  readonly closed = output<void>();

  protected onEscape(): void {
    if (this.open()) {
      this.closed.emit();
    }
  }
}
