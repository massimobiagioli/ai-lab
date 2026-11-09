import { Component, input } from '@angular/core';

@Component({
  selector: 'app-progress',
  template: `
    <div
      class="ui-progress"
      role="progressbar"
      [attr.aria-valuenow]="value()"
      aria-valuemin="0"
      aria-valuemax="100"
      [attr.aria-label]="label()"
    >
      <span class="ui-progress__bar" [style.width.%]="value()"></span>
    </div>
  `,
  styles: `
    .ui-progress {
      height: 8px;
      border: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface-sunken);
    }
    .ui-progress__bar {
      display: block;
      height: 100%;
      background: var(--ailab-color-accent);
      transition: width var(--ailab-motion-base) steps(8);
    }
  `,
})
export class Progress {
  readonly value = input.required<number>();
  readonly label = input('Progress');
}
