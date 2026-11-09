import { Component, input, output } from '@angular/core';
import { Icon, type IconName } from '../icon/icon';

@Component({
  selector: 'app-pixel-icon-button',
  imports: [Icon],
  template: `
    <button
      class="pixel-icon-button"
      type="button"
      [attr.aria-label]="label()"
      (click)="pressed.emit($event)"
    >
      <app-icon [name]="icon()" />
    </button>
  `,
  styles: `
    .pixel-icon-button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 32px;
      height: 32px;
      border: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text);
      cursor: pointer;
    }
    .pixel-icon-button:hover {
      background: var(--ailab-color-surface-raised);
    }
    .pixel-icon-button:focus-visible {
      outline: 2px solid var(--ailab-color-accent);
      outline-offset: 2px;
    }
  `,
})
export class PixelIconButton {
  readonly icon = input.required<IconName>();
  readonly label = input.required<string>();
  readonly pressed = output<MouseEvent>();
}
