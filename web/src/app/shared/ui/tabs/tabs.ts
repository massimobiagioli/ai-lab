import { Component, ElementRef, input, model, viewChildren } from '@angular/core';

export interface TabItem {
  readonly id: string;
  readonly label: string;
}

@Component({
  selector: 'app-tabs',
  template: `
    <div
      class="ui-tabs"
      role="tablist"
      aria-orientation="horizontal"
      tabindex="-1"
      [attr.aria-label]="label()"
      (keydown)="onKey($event)"
    >
      @for (tab of tabs(); track tab.id) {
        <button
          #tabButton
          class="ui-tabs__tab"
          role="tab"
          type="button"
          [class.is-active]="active() === tab.id"
          [attr.aria-selected]="active() === tab.id"
          [attr.tabindex]="active() === tab.id ? 0 : -1"
          (click)="select(tab.id)"
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
      border-bottom: 1px solid var(--ailab-color-border);
      margin-bottom: var(--ailab-space-4);
    }
    .ui-tabs__tab {
      padding: 0.65rem 0.9rem;
      border: 0;
      border-bottom: 2px solid transparent;
      background: transparent;
      color: var(--ailab-color-muted);
      font-family: var(--ailab-font-sans);
      font-size: var(--ailab-font-size-md);
      font-weight: 600;
      cursor: pointer;
    }
    .ui-tabs__tab.is-active {
      color: var(--ailab-color-accent);
      border-bottom-color: var(--ailab-color-accent);
    }
    .ui-tabs__tab:focus-visible {
      outline: none;
      box-shadow: inset 0 0 0 2px var(--ailab-color-accent-soft);
    }
  `,
})
export class Tabs {
  readonly label = input.required<string>();
  readonly tabs = input.required<readonly TabItem[]>();
  readonly active = model.required<string>();
  private readonly buttons = viewChildren<ElementRef<HTMLButtonElement>>('tabButton');

  protected select(id: string): void {
    this.active.set(id);
  }

  protected onKey(event: KeyboardEvent): void {
    const tabs = this.tabs();
    const current = Math.max(
      0,
      tabs.findIndex((tab) => tab.id === this.active()),
    );
    let next!: number;
    switch (event.key) {
      case 'ArrowRight':
        next = Math.min(tabs.length - 1, current + 1);
        break;
      case 'ArrowLeft':
        next = Math.max(0, current - 1);
        break;
      case 'Home':
        next = 0;
        break;
      case 'End':
        next = Math.max(0, tabs.length - 1);
        break;
      default:
        return;
    }
    event.preventDefault();
    const target = tabs[next];
    if (target !== undefined) {
      this.active.set(target.id);
      this.buttons()[next]?.nativeElement.focus();
    }
  }
}
