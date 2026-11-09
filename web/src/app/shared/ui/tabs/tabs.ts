import { Component, input, model } from '@angular/core';

export interface TabItem {
  readonly id: string;
  readonly label: string;
}

@Component({
  selector: 'app-tabs',
  template: `
    <div class="ui-tabs" role="tablist" [attr.aria-label]="label()">
      @for (tab of tabs(); track tab.id) {
        <button
          class="ui-tabs__tab"
          role="tab"
          type="button"
          [class.is-active]="active() === tab.id"
          [attr.aria-selected]="active() === tab.id"
          (click)="active.set(tab.id)"
        >
          {{ tab.label }}
        </button>
      }
    </div>
  `,
  styles: `
    .ui-tabs {
      display: flex;
      gap: var(--ailab-space-1);
      border-bottom: 2px solid var(--ailab-color-muted);
      margin-bottom: var(--ailab-space-4);
    }
    .ui-tabs__tab {
      padding: var(--ailab-space-2) var(--ailab-space-3);
      border: 2px solid transparent;
      border-bottom: 0;
      background: transparent;
      color: var(--ailab-color-muted);
      font-family: var(--ailab-font-mono);
      font-size: var(--ailab-font-size-md);
      cursor: pointer;
    }
    .ui-tabs__tab.is-active {
      color: var(--ailab-color-accent);
      border-color: var(--ailab-color-accent);
    }
    .ui-tabs__tab:focus-visible {
      outline: 2px solid var(--ailab-color-accent);
      outline-offset: 2px;
    }
  `,
})
export class Tabs {
  readonly label = input.required<string>();
  readonly tabs = input.required<readonly TabItem[]>();
  readonly active = model.required<string>();
}
