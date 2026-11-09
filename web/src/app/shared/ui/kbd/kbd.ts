import { Component } from '@angular/core';

@Component({
  selector: 'app-kbd',
  template: `<kbd class="ui-kbd"><ng-content /></kbd>`,
  styles: `
    .ui-kbd {
      display: inline-block;
      padding: 0 var(--ailab-space-1);
      border: 1px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface-sunken);
      font-family: var(--ailab-font-mono);
      font-size: var(--ailab-font-size-sm);
    }
  `,
})
export class Kbd {}
