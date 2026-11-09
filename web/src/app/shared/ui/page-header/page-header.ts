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
      align-items: flex-end;
      justify-content: space-between;
      gap: var(--ailab-space-3);
      margin-bottom: var(--ailab-space-4);
    }
    .ui-page-header__title {
      margin: 0;
      font-size: var(--ailab-font-size-xl);
      letter-spacing: 0.04em;
    }
    .ui-page-header__subtitle {
      margin: var(--ailab-space-1) 0 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-md);
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
