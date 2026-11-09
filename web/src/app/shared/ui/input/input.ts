import { Component, input, model } from '@angular/core';

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
      color: var(--ailab-color-muted);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-size: var(--ailab-font-size-sm);
    }
    .ui-input__control {
      padding: var(--ailab-space-2);
      border: 2px solid var(--ailab-color-muted);
      border-radius: var(--ailab-radius);
      background: var(--ailab-color-surface-sunken);
      color: var(--ailab-color-text);
      font-family: var(--ailab-font-mono);
      font-size: var(--ailab-font-size-md);
    }
    .ui-input__control:focus-visible {
      outline: 2px solid var(--ailab-color-accent);
      outline-offset: 2px;
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
  readonly type = input<'text' | 'search' | 'email'>('text');
  readonly placeholder = input('');
  readonly autocomplete = input<string | null>(null);
  readonly error = input<string | null>(null);
  readonly disabled = input(false);
  protected readonly errorId = `input-error-${crypto.randomUUID()}`;

  protected onInput(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLInputElement) {
      this.value.set(target.value);
    }
  }
}
