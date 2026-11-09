import { Component, input } from '@angular/core';

@Component({
  selector: 'app-card',
  template: `
    <article class="ui-card" [attr.aria-label]="label() ?? null">
      @if (title() !== null) {
        <h3 class="ui-card__title">{{ title() }}</h3>
      }
      <ng-content />
    </article>
  `,
  styles: `
    .ui-card {
      border: 2px solid var(--ailab-color-muted);
      border-radius: var(--ailab-radius);
      background: var(--ailab-color-surface);
      padding: var(--ailab-space-4);
      box-shadow: 4px 4px 0 var(--ailab-color-bg);
    }
    .ui-card__title {
      margin: 0 0 var(--ailab-space-3);
      font-size: var(--ailab-font-size-lg);
      letter-spacing: 0.04em;
    }
  `,
})
export class Card {
  readonly title = input<string | null>(null);
  readonly label = input<string | null>(null);
}
