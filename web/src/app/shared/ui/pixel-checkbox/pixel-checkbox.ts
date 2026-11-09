import { Component, input, model } from '@angular/core';

@Component({
  selector: 'app-pixel-checkbox',
  template: `
    <label class="pixel-checkbox">
      <input
        class="pixel-checkbox__control"
        type="checkbox"
        [checked]="checked()"
        (change)="onChange($event)"
      />
      <span>{{ label() }}</span>
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
  `,
})
export class PixelCheckbox {
  readonly label = input.required<string>();
  readonly checked = model(false);

  protected onChange(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLInputElement) {
      this.checked.set(target.checked);
    }
  }
}
