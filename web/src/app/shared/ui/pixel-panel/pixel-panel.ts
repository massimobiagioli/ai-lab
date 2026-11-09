import { Component, input } from '@angular/core';

@Component({
  selector: 'app-pixel-panel',
  template: `
    <section
      class="pixel-panel"
      [class.pixel-panel--raised]="raised()"
      [attr.aria-label]="label() ?? null"
    >
      @if (title() !== null) {
        <h3 class="pixel-panel__title">{{ title() }}</h3>
      }
      <ng-content />
    </section>
  `,
  styles: `
    .pixel-panel {
      border: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface);
      padding: var(--ailab-space-4);
    }
    .pixel-panel--raised {
      box-shadow: var(--ailab-shadow-hard);
    }
    .pixel-panel__title {
      margin: 0 0 var(--ailab-space-3);
      font-size: var(--ailab-font-size-lg);
      letter-spacing: 0.04em;
    }
  `,
})
export class PixelPanel {
  readonly title = input<string | null>(null);
  readonly label = input<string | null>(null);
  readonly raised = input(false);
}
