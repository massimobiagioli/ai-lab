import { Component, input, output } from '@angular/core';

export type ButtonVariant = 'primary' | 'ghost' | 'danger';

@Component({
  selector: 'app-button',
  template: `
    <button
      [class]="'ui-button ui-button--' + variant()"
      [type]="type()"
      [disabled]="disabled() || loading()"
      [attr.aria-busy]="loading()"
      (click)="pressed.emit($event)"
    >
      @if (loading()) {
        <span aria-hidden="true">…</span>
      }
      <ng-content />
    </button>
  `,
  styles: `
    .ui-button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: var(--ailab-space-2);
      min-height: 2.375rem;
      padding: 0.55rem 0.95rem;
      border: 1px solid var(--ailab-color-border);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text-soft);
      font-family: var(--ailab-font-sans);
      font-size: var(--ailab-font-size-md);
      font-weight: 600;
      cursor: pointer;
      transition:
        background var(--ailab-motion-fast) ease,
        border-color var(--ailab-motion-fast) ease,
        box-shadow var(--ailab-motion-fast) ease,
        transform var(--ailab-motion-fast) ease;
    }
    .ui-button:hover:not(:disabled) {
      background: var(--ailab-color-surface-raised);
      border-color: var(--ailab-color-border-strong);
    }
    .ui-button:active:not(:disabled) {
      transform: translateY(1px);
    }
    .ui-button:focus-visible {
      outline: none;
      box-shadow: 0 0 0 3px var(--ailab-color-accent-soft);
      border-color: var(--ailab-color-accent);
    }
    .ui-button:disabled {
      opacity: 0.55;
      cursor: not-allowed;
    }
    .ui-button--primary {
      border-color: transparent;
      background: var(--ailab-color-accent);
      color: #ffffff;
      box-shadow: 0 8px 18px -12px var(--ailab-color-accent);
    }
    .ui-button--primary:hover:not(:disabled) {
      background: var(--ailab-color-accent-strong);
      border-color: transparent;
    }
    .ui-button--danger {
      border-color: transparent;
      background: var(--ailab-color-danger-soft);
      color: var(--ailab-color-danger);
    }
    .ui-button--danger:hover:not(:disabled) {
      background: #fecaca;
      border-color: transparent;
    }
    .ui-button[aria-busy='true'] {
      cursor: progress;
    }
  `,
})
export class Button {
  readonly variant = input<ButtonVariant>('ghost');
  readonly type = input<'button' | 'submit'>('button');
  readonly disabled = input(false);
  readonly loading = input(false);
  readonly pressed = output<MouseEvent>();
}
