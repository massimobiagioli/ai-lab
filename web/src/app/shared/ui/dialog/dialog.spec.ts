import { TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { Dialog } from './dialog';

describe('Dialog', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [Dialog] }).compileComponents();
  });

  it('closes on Escape', () => {
    const fixture = TestBed.createComponent(Dialog);
    fixture.componentRef.setInput('label', 'Confirm');
    fixture.componentRef.setInput('title', 'Confirm');
    fixture.componentRef.setInput('open', true);
    const closed = vi.fn();
    fixture.componentInstance.closed.subscribe(closed);
    fixture.detectChanges();
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }));
    expect(closed).toHaveBeenCalled();
  });
});
