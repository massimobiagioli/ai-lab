import { Component, input } from '@angular/core';

@Component({
  selector: 'app-empty-state',
  template: `
    <div class="ui-empty" role="status">
      <p class="ui-empty__title">{{ title() }}</p>
      <p class="ui-empty__message">{{ message() }}</p>
      <ng-content />
    </div>
  `,
  styles: `
    .ui-empty {
      border: 1px dashed var(--ailab-color-border-strong);
      border-radius: var(--ailab-radius);
      background: var(--ailab-color-surface);
      padding: var(--ailab-space-10) var(--ailab-space-6);
      text-align: center;
      color: var(--ailab-color-muted);
    }
    .ui-empty__title {
      margin: 0 0 var(--ailab-space-2);
      color: var(--ailab-color-text);
      font-size: 1.05rem;
      font-weight: 650;
    }
    .ui-empty__message {
      margin: 0;
      font-size: var(--ailab-font-size-md);
    }
  `,
})
export class EmptyState {
  readonly title = input.required<string>();
  readonly message = input.required<string>();
}
