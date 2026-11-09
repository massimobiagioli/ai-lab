import { Component } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { KeyboardDnd, MoveEvent } from './keyboard-dnd';

@Component({
  imports: [KeyboardDnd],
  template: `<div appKeyboardDnd [index]="1" [count]="3" (moved)="moves.push($event)"></div>`,
})
class Host {
  readonly moves: MoveEvent[] = [];
}

describe('KeyboardDnd', () => {
  it('emits a move on Alt+Arrow', () => {
    const fixture = TestBed.createComponent(Host);
    fixture.detectChanges();
    const element = fixture.nativeElement.querySelector('div') as HTMLElement;
    element.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft', altKey: true }));
    fixture.detectChanges();
    expect(fixture.componentInstance.moves).toEqual([{ from: 1, to: 0 }]);
  });

  it('ignores arrows without Alt', () => {
    const fixture = TestBed.createComponent(Host);
    fixture.detectChanges();
    const element = fixture.nativeElement.querySelector('div') as HTMLElement;
    element.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
    expect(fixture.componentInstance.moves).toEqual([]);
  });
});
