import { Component } from '@angular/core';
import { Shell } from './layout/shell/shell';

@Component({
  selector: 'app-root',
  imports: [Shell],
  template: '<app-shell />',
  styles: `
    :host {
      display: block;
      min-height: 100dvh;
      background: var(--ailab-color-bg);
      color: var(--ailab-color-text);
    }
  `,
})
export class App {}
