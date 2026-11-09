import { Component, input, output } from '@angular/core';
import { Button } from '../button/button';
import { FocusTrap } from '../../directives/focus-trap';

@Component({
  selector: 'app-dialog',
  host: { '(document:keydown.escape)': 'onEscape()' },
  imports: [Button, FocusTrap],
  template: `
    @if (open()) {
      <div
        class="ui-dialog"
        role="dialog"
        aria-modal="true"
        [attr.aria-label]="label()"
        appFocusTrap
      >
        <h2 class="ui-dialog__title">{{ title() }}</h2>
        <div class="ui-dialog__body"><ng-content /></div>
        <div class="ui-dialog__actions">
          <app-button variant="primary" (pressed)="confirmed.emit()">Confirm</app-button>
          <app-button (pressed)="closed.emit()">Cancel</app-button>
        </div>
      </div>
      <button
        class="ui-dialog__scrim"
        type="button"
        aria-label="Close dialog"
        (click)="closed.emit()"
      ></button>
    }
  `,
  styles: `
    .ui-dialog {
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      width: min(480px, 92vw);
      background: var(--ailab-color-surface);
      border: 2px solid var(--ailab-color-accent);
      padding: var(--ailab-space-4);
      z-index: var(--ailab-z-toast);
    }
    .ui-dialog__title {
      margin: 0 0 var(--ailab-space-3);
      font-size: var(--ailab-font-size-lg);
    }
    .ui-dialog__actions {
      display: flex;
      gap: var(--ailab-space-2);
      margin-top: var(--ailab-space-4);
    }
    .ui-dialog__scrim {
      position: fixed;
      inset: 0;
      background: var(--ailab-color-overlay);
      border: 0;
      z-index: calc(var(--ailab-z-toast) - 1);
    }
  `,
})
export class Dialog {
  readonly label = input.required<string>();
  readonly title = input.required<string>();
  readonly open = input.required<boolean>();
  readonly closed = output<void>();
  readonly confirmed = output<void>();

  protected onEscape(): void {
    if (this.open()) {
      this.closed.emit();
    }
  }
}
