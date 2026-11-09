import { Component, input } from '@angular/core';

/** Explicit degraded/partial-data banner used when the edge read model is unavailable. */
@Component({
  selector: 'app-degraded',
  template: `
    @if (active()) {
      <p class="degraded" role="status"><strong>Degraded:</strong> {{ reason() }}</p>
    }
  `,
  styles: `
    .degraded {
      margin: 0 0 var(--ailab-space-3);
      padding: var(--ailab-space-2);
      border: 2px dashed var(--ailab-color-warning);
      color: var(--ailab-color-warning);
      font-size: var(--ailab-font-size-sm);
    }
  `,
})
export class Degraded {
  readonly active = input.required<boolean>();
  readonly reason = input('showing demo data until the edge API is available');
}
