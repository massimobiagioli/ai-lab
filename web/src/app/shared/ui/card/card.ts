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
      border: var(--ailab-border);
      border-radius: var(--ailab-radius);
      background: var(--ailab-color-surface);
      padding: clamp(1rem, 1.6vw, 1.5rem);
      box-shadow: var(--ailab-shadow-card);
    }
    .ui-card__title {
      margin: 0 0 var(--ailab-space-3);
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-lg);
      font-weight: 650;
      letter-spacing: -0.015em;
    }
  `,
})
export class Card {
  readonly title = input<string | null>(null);
  readonly label = input<string | null>(null);
}
