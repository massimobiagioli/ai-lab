import { Component, input, output } from '@angular/core';
import { Button } from '../button/button';

@Component({
  selector: 'app-pagination',
  imports: [Button],
  template: `
    <nav class="ui-pagination" aria-label="Pagination">
      <app-button [disabled]="!hasPrevious()" (pressed)="previous.emit()">Previous</app-button>
      <span class="ui-pagination__status">{{ label() }}</span>
      <app-button [disabled]="!hasNext()" (pressed)="next.emit()">Next</app-button>
    </nav>
  `,
  styles: `
    .ui-pagination {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-3);
    }
    .ui-pagination__status {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
  `,
})
export class Pagination {
  readonly label = input.required<string>();
  readonly hasPrevious = input(false);
  readonly hasNext = input(false);
  readonly previous = output<void>();
  readonly next = output<void>();
}
