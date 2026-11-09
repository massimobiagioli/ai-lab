import { Component, input } from '@angular/core';

@Component({
  selector: 'app-spinner',
  template: `
    <span class="ui-spinner" role="status" [attr.aria-label]="label()">
      <span class="ui-spinner__frame" aria-hidden="true"></span>
    </span>
  `,
  styles: `
    .ui-spinner {
      display: inline-flex;
      align-items: center;
      gap: var(--ailab-space-2);
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .ui-spinner__frame {
      width: 12px;
      height: 12px;
      border: 2px solid var(--ailab-color-muted);
      border-top-color: var(--ailab-color-accent);
      animation: ui-spin 0.8s steps(8) infinite;
    }
    @keyframes ui-spin {
      to {
        transform: rotate(360deg);
      }
    }
    @media (prefers-reduced-motion: reduce) {
      .ui-spinner__frame {
        animation: none;
      }
    }
  `,
})
export class Spinner {
  readonly label = input('Loading');
}
