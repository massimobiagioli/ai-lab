import { Component, input, model } from '@angular/core';

@Component({
  selector: 'app-pixel-checkbox',
  template: `
    <label class="pixel-checkbox">
      <input
        class="pixel-checkbox__control"
        type="checkbox"
        [checked]="checked()"
        [disabled]="disabled()"
        [attr.aria-invalid]="error() !== null"
        [attr.aria-describedby]="error() !== null ? errorId : null"
        (change)="onChange($event)"
      />
      <span>{{ label() }}</span>
      @if (error() !== null) {
        <span class="pixel-checkbox__error" [id]="errorId" role="alert">{{ error() }}</span>
      }
    </label>
  `,
  styles: `
    .pixel-checkbox {
      display: inline-flex;
      align-items: center;
      gap: var(--ailab-space-2);
      font-size: var(--ailab-font-size-md);
    }
    .pixel-checkbox__control {
      width: 16px;
      height: 16px;
      accent-color: var(--ailab-color-accent);
    }
    .pixel-checkbox__error {
      color: var(--ailab-color-danger);
      font-size: var(--ailab-font-size-sm);
    }
  `,
})
export class PixelCheckbox {
  readonly label = input.required<string>();
  readonly checked = model(false);
  readonly disabled = input(false);
  readonly error = input<string | null>(null);
  protected readonly errorId = `checkbox-error-${crypto.randomUUID()}`;

  protected onChange(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLInputElement) {
      this.checked.set(target.checked);
    }
  }
}
