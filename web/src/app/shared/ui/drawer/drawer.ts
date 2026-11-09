import { Component, input, output } from '@angular/core';
import { Button } from '../button/button';
import { FocusTrap } from '../../directives/focus-trap';

@Component({
  selector: 'app-drawer',
  imports: [Button, FocusTrap],
  template: `
    @if (open()) {
      <div class="drawer" role="dialog" aria-modal="true" [attr.aria-label]="label()" appFocusTrap>
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
      width: min(320px, 90vw);
      background: var(--ailab-color-surface);
      border-right: 2px solid var(--ailab-color-accent);
      padding: var(--ailab-space-4);
      z-index: var(--ailab-z-drawer);
      overflow-y: auto;
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
      font-size: var(--ailab-font-size-lg);
      letter-spacing: 0.04em;
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
  readonly closed = output<void>();
}
