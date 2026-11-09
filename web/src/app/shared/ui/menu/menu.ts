import { Component, ElementRef, input, output, signal, viewChildren } from '@angular/core';

export interface MenuItem {
  readonly id: string;
  readonly label: string;
}

@Component({
  selector: 'app-menu',
  template: `
    <ul
      class="ui-menu"
      role="menu"
      tabindex="-1"
      [attr.aria-label]="label()"
      (keydown)="onKey($event)"
    >
      @for (item of items(); track item.id; let index = $index) {
        <li role="none">
          <button
            #menuItem
            class="ui-menu__item"
            role="menuitem"
            type="button"
            [attr.tabindex]="index === activeIndex() ? 0 : -1"
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
  readonly closed = output<void>();
  protected readonly activeIndex = signal(0);
  private readonly buttons = viewChildren<ElementRef<HTMLButtonElement>>('menuItem');

  protected onKey(event: KeyboardEvent): void {
    const items = this.items();
    const current = this.activeIndex();
    let next!: number;
    switch (event.key) {
      case 'ArrowDown':
        next = Math.min(items.length - 1, current + 1);
        break;
      case 'ArrowUp':
        next = Math.max(0, current - 1);
        break;
      case 'Home':
        next = 0;
        break;
      case 'End':
        next = Math.max(0, items.length - 1);
        break;
      case 'Escape':
        event.preventDefault();
        this.closed.emit();
        return;
      default:
        return;
    }
    event.preventDefault();
    this.activeIndex.set(next);
    this.buttons()[next]?.nativeElement.focus();
  }
}
