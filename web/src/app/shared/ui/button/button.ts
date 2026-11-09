import { Component, input, output } from '@angular/core';

export type ButtonVariant = 'primary' | 'ghost' | 'danger';

@Component({
  selector: 'app-button',
  template: `
    <button
      [class]="'ui-button ui-button--' + variant()"
      [type]="type()"
      [disabled]="disabled()"
      (click)="pressed.emit($event)"
    >
      <ng-content />
    </button>
  `,
  styles: `
    .ui-button {
      display: inline-flex;
      align-items: center;
      gap: var(--ailab-space-2);
      padding: var(--ailab-space-2) var(--ailab-space-3);
      border: 2px solid var(--ailab-color-muted);
      border-radius: var(--ailab-radius);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text);
      font-family: var(--ailab-font-mono);
      font-size: var(--ailab-font-size-md);
      cursor: pointer;
      transition:
        transform 80ms ease,
        background 160ms ease;
    }
    .ui-button:hover:not(:disabled) {
      background: var(--ailab-color-surface-raised);
    }
    .ui-button:active:not(:disabled) {
      transform: translateY(1px);
    }
    .ui-button:focus-visible {
      outline: 2px solid var(--ailab-color-accent);
      outline-offset: 2px;
    }
    .ui-button:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
    .ui-button--primary {
      border-color: var(--ailab-color-accent);
      color: var(--ailab-color-accent);
    }
    .ui-button--danger {
      border-color: var(--ailab-color-danger);
      color: var(--ailab-color-danger);
    }
  `,
})
export class Button {
  readonly variant = input<ButtonVariant>('ghost');
  readonly type = input<'button' | 'submit'>('button');
  readonly disabled = input(false);
  readonly pressed = output<MouseEvent>();
}
