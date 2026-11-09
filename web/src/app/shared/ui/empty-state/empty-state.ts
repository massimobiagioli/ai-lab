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
      border: 2px dashed var(--ailab-color-muted);
      border-radius: var(--ailab-radius);
      padding: var(--ailab-space-6);
      text-align: center;
      color: var(--ailab-color-muted);
    }
    .ui-empty__title {
      margin: 0 0 var(--ailab-space-2);
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-lg);
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
