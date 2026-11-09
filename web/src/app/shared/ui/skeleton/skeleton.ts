import { Component, input } from '@angular/core';

@Component({
  selector: 'app-skeleton',
  template: `<span
    class="ui-skeleton"
    [style.width]="width()"
    [style.height]="height()"
    aria-hidden="true"
  ></span>`,
  styles: `
    .ui-skeleton {
      display: inline-block;
      background: var(--ailab-color-surface-raised);
      animation: ui-pulse 1.2s steps(2) infinite;
    }
    @keyframes ui-pulse {
      50% {
        opacity: 0.5;
      }
    }
    @media (prefers-reduced-motion: reduce) {
      .ui-skeleton {
        animation: none;
      }
    }
  `,
})
export class Skeleton {
  readonly width = input('100%');
  readonly height = input('1rem');
}
