import { Component, input, model } from '@angular/core';

export interface SelectOption {
  readonly value: string;
  readonly label: string;
}

@Component({
  selector: 'app-select',
  template: `
    <label class="ui-select">
      <span class="ui-select__label">{{ label() }}</span>
      <select class="ui-select__control" [value]="value()" (change)="onChange($event)">
        @for (option of options(); track option.value) {
          <option [value]="option.value">{{ option.label }}</option>
        }
      </select>
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
      color: var(--ailab-color-muted);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-size: var(--ailab-font-size-sm);
    }
    .ui-select__control {
      padding: var(--ailab-space-2);
      border: 2px solid var(--ailab-color-muted);
      border-radius: var(--ailab-radius);
      background: var(--ailab-color-surface-sunken);
      color: var(--ailab-color-text);
      font-family: var(--ailab-font-mono);
    }
  `,
})
export class Select {
  readonly label = input.required<string>();
  readonly options = input.required<readonly SelectOption[]>();
  readonly value = model('');

  protected onChange(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLSelectElement) {
      this.value.set(target.value);
    }
  }
}
