import { Component, input, model } from '@angular/core';
import { createId } from '../../../core/ids';

export interface SelectOption {
  readonly value: string;
  readonly label: string;
}

@Component({
  selector: 'app-select',
  template: `
    <label class="ui-select">
      <span class="ui-select__label">{{ label() }}</span>
      <select
        class="ui-select__control"
        [value]="value()"
        [disabled]="disabled()"
        [attr.aria-invalid]="error() !== null"
        [attr.aria-describedby]="error() !== null ? errorId : null"
        (change)="onChange($event)"
      >
        @for (option of options(); track option.value) {
          <option [value]="option.value" [selected]="option.value === value()">
            {{ option.label }}
          </option>
        }
      </select>
      @if (error() !== null) {
        <span class="ui-select__error" [id]="errorId" role="alert">{{ error() }}</span>
      }
    </label>
  `,
  styles: `
    .ui-select {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-1);
      font-size: var(--ailab-font-size-md);
    }
    .ui-select__label {
      color: var(--ailab-color-text-soft);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
    }
    .ui-select__control:disabled {
      opacity: 0.5;
    }
    .ui-select__error {
      color: var(--ailab-color-danger);
      font-size: var(--ailab-font-size-sm);
    }
    .ui-select__control {
      padding: 0.65rem 0.8rem;
      border: 1px solid var(--ailab-color-border);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text);
      font-family: var(--ailab-font-sans);
    }
    .ui-select__control:focus-visible {
      outline: none;
      border-color: var(--ailab-color-accent);
      box-shadow: 0 0 0 3px var(--ailab-color-accent-soft);
    }
  `,
})
export class Select {
  readonly label = input.required<string>();
  readonly options = input.required<readonly SelectOption[]>();
  readonly value = model('');
  readonly disabled = input(false);
  readonly error = input<string | null>(null);
  protected readonly errorId = `select-error-${createId()}`;

  protected onChange(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLSelectElement) {
      this.value.set(target.value);
    }
  }
}
