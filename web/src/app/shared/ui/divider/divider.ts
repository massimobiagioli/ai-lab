import { Component } from '@angular/core';

@Component({
  selector: 'app-divider',
  template: `<hr class="ui-divider" />`,
  styles: `
    .ui-divider {
      border: 0;
      border-top: 2px solid var(--ailab-color-surface-raised);
      margin: var(--ailab-space-3) 0;
    }
  `,
})
export class Divider {}
