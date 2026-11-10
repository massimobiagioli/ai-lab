import { Component, input } from '@angular/core';

@Component({
  selector: 'app-page-header',
  template: `
    <header class="ui-page-header">
      <div>
        <h1 class="ui-page-header__title">{{ title() }}</h1>
        @if (subtitle() !== null) {
          <p class="ui-page-header__subtitle">{{ subtitle() }}</p>
        }
      </div>
      <div class="ui-page-header__actions"><ng-content /></div>
    </header>
  `,
  styles: `
    .ui-page-header {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-4);
      margin-bottom: var(--ailab-space-6);
    }
    .ui-page-header__title {
      margin: 0;
      color: var(--ailab-color-text);
      font-size: clamp(1.55rem, 3vw, var(--ailab-font-size-xl));
      font-weight: 720;
      letter-spacing: -0.035em;
    }
    .ui-page-header__subtitle {
      margin: var(--ailab-space-1) 0 0;
      color: var(--ailab-color-muted);
      font-size: 0.95rem;
    }
    .ui-page-header__actions {
      display: flex;
      gap: var(--ailab-space-2);
    }
  `,
})
export class PageHeader {
  readonly title = input.required<string>();
  readonly subtitle = input<string | null>(null);
}
