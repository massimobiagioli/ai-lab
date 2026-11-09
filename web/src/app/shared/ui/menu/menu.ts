import { Component, input, output } from '@angular/core';

export interface MenuItem {
  readonly id: string;
  readonly label: string;
}

@Component({
  selector: 'app-menu',
  template: `
    <ul class="ui-menu" role="menu" [attr.aria-label]="label()">
      @for (item of items(); track item.id) {
        <li role="none">
          <button
            class="ui-menu__item"
            role="menuitem"
            type="button"
            (click)="selected.emit(item.id)"
          >
            {{ item.label }}
          </button>
        </li>
      }
    </ul>
  `,
  styles: `
    .ui-menu {
      list-style: none;
      margin: 0;
      padding: 0;
      border: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface);
      min-width: 160px;
    }
    .ui-menu__item {
      display: block;
      width: 100%;
      padding: var(--ailab-space-2);
      text-align: left;
      border: 0;
      background: transparent;
      color: var(--ailab-color-text);
      font-family: var(--ailab-font-mono);
      cursor: pointer;
    }
    .ui-menu__item:hover,
    .ui-menu__item:focus-visible {
      background: var(--ailab-color-surface-raised);
      outline: none;
    }
  `,
})
export class Menu {
  readonly label = input.required<string>();
  readonly items = input.required<readonly MenuItem[]>();
  readonly selected = output<string>();
}
