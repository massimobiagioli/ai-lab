import { Component, input, model } from '@angular/core';
import { createId } from '../../../core/ids';

@Component({
  selector: 'app-input',
  template: `
    <label class="ui-input">
      <span class="ui-input__label">{{ label() }}</span>
      <input
        class="ui-input__control"
        [type]="type()"
        [value]="value()"
        [placeholder]="placeholder()"
        [attr.autocomplete]="autocomplete()"
        [disabled]="disabled()"
        [attr.aria-invalid]="error() !== null"
        [attr.aria-describedby]="error() !== null ? errorId : null"
        (input)="onInput($event)"
      />
      @if (error() !== null) {
        <span class="ui-input__error" [id]="errorId" role="alert">{{ error() }}</span>
      }
    </label>
  `,
  styles: `
    .ui-input {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-1);
      font-size: var(--ailab-font-size-md);
    }
    .ui-input__label {
      color: var(--ailab-color-text-soft);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
    }
    .ui-input__control {
      padding: 0.65rem 0.8rem;
      border: 1px solid var(--ailab-color-border);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text);
      font-family: var(--ailab-font-sans);
      font-size: var(--ailab-font-size-md);
      transition:
        border-color var(--ailab-motion-fast) ease,
        box-shadow var(--ailab-motion-fast) ease;
    }
    .ui-input__control:focus-visible {
      outline: none;
      border-color: var(--ailab-color-accent);
      box-shadow: 0 0 0 3px var(--ailab-color-accent-soft);
    }
    .ui-input__error {
      color: var(--ailab-color-danger);
      font-size: var(--ailab-font-size-sm);
    }
    .ui-input__control:disabled {
      opacity: 0.5;
    }
  `,
})
export class Input {
  readonly label = input.required<string>();
  readonly value = model('');
  readonly type = input<'text' | 'search' | 'email' | 'password'>('text');
  readonly placeholder = input('');
  readonly autocomplete = input<string | null>(null);
  readonly error = input<string | null>(null);
  readonly disabled = input(false);
  protected readonly errorId = `input-error-${createId()}`;

  protected onInput(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLInputElement) {
      this.value.set(target.value);
    }
  }
}
