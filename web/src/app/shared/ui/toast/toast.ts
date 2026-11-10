import { Component, input } from '@angular/core';
import type { StatusTone } from '../../status-badge/status-badge';

@Component({
  selector: 'app-toast',
  template: `
    <div class="ui-toast" [class]="'ui-toast--' + tone()" role="status">
      {{ message() }}
    </div>
  `,
  styles: `
    .ui-toast {
      padding: var(--ailab-space-2) var(--ailab-space-3);
      border: var(--ailab-border);
      border-radius: var(--ailab-radius-md);
      box-shadow: var(--ailab-shadow-card);
      background: var(--ailab-color-surface);
    }
    .ui-toast--success {
      border-color: var(--ailab-color-success);
      color: var(--ailab-color-success);
    }
    .ui-toast--warning {
      border-color: var(--ailab-color-warning);
      color: var(--ailab-color-warning);
    }
    .ui-toast--danger {
      border-color: var(--ailab-color-danger);
      color: var(--ailab-color-danger);
    }
  `,
})
export class Toast {
  readonly message = input.required<string>();
  readonly tone = input<StatusTone>('neutral');
}
