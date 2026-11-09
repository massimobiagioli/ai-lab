import { Component, input, model } from '@angular/core';

@Component({
  selector: 'app-pixel-textarea',
  template: `
    <label class="pixel-textarea">
      <span class="pixel-textarea__label">{{ label() }}</span>
      <textarea
        class="pixel-textarea__control"
        [value]="value()"
        [attr.aria-invalid]="error() !== null"
        [rows]="rows()"
        (input)="onInput($event)"
      ></textarea>
      @if (error() !== null) {
        <span class="pixel-textarea__error" role="alert">{{ error() }}</span>
      }
    </label>
  `,
  styles: `
    .pixel-textarea {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-1);
      font-size: var(--ailab-font-size-md);
    }
    .pixel-textarea__label {
      color: var(--ailab-color-muted);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      font-size: var(--ailab-font-size-sm);
    }
    .pixel-textarea__control {
      padding: var(--ailab-space-2);
      border: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface-sunken);
      color: var(--ailab-color-text);
      font-family: var(--ailab-font-mono);
      resize: vertical;
    }
    .pixel-textarea__error {
      color: var(--ailab-color-danger);
      font-size: var(--ailab-font-size-sm);
    }
  `,
})
export class PixelTextarea {
  readonly label = input.required<string>();
  readonly value = model('');
  readonly rows = input(3);
  readonly error = input<string | null>(null);

  protected onInput(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLTextAreaElement) {
      this.value.set(target.value);
    }
  }
}
