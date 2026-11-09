import { Directive, input, output } from '@angular/core';

export interface MoveEvent {
  readonly from: number;
  readonly to: number;
}

/** Keyboard reordering: Alt+ArrowLeft/Right emits a move event. */
@Directive({
  selector: '[appKeyboardDnd]',
  host: {
    '[attr.tabindex]': '0',
    '(keydown)': 'onKey($event)',
  },
})
export class KeyboardDnd {
  readonly index = input.required<number>();
  readonly count = input.required<number>();
  readonly moved = output<MoveEvent>();

  protected onKey(event: KeyboardEvent): void {
    if (!event.altKey) {
      return;
    }
    const index = this.index();
    if (event.key === 'ArrowLeft' && index > 0) {
      event.preventDefault();
      this.moved.emit({ from: index, to: index - 1 });
    } else if (event.key === 'ArrowRight' && index < this.count() - 1) {
      event.preventDefault();
      this.moved.emit({ from: index, to: index + 1 });
    }
  }
}
