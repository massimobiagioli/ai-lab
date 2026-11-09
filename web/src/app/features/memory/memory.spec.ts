import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { vi } from 'vitest';
import { DEMO_MEMORY } from '../../core/demo/demo-data';
import { Memory } from './memory';
import { MemoryStore } from './store/memory.store';

function fakeStore() {
  return {
    state: signal('ready'),
    results: signal(DEMO_MEMORY),
    error: signal(null),
    demo: signal(true),
    pinned: signal<ReadonlySet<string>>(new Set()),
    search: vi.fn(),
    remove: vi.fn(),
    togglePin: vi.fn(),
  };
}

describe('Memory', () => {
  it('renders the notes and performs a search', async () => {
    const store = fakeStore();
    await TestBed.configureTestingModule({
      imports: [Memory],
      providers: [provideRouter([]), { provide: MemoryStore, useValue: store }],
    }).compileComponents();
    const fixture = TestBed.createComponent(Memory);
    fixture.detectChanges();
    const element = fixture.nativeElement as HTMLElement;
    expect(element.querySelectorAll('[data-testid="memory-note"]').length).toBe(DEMO_MEMORY.length);
    const input = element.querySelector<HTMLInputElement>('input');
    if (input) {
      input.value = 'board';
      input.dispatchEvent(new Event('input'));
    }
    fixture.detectChanges();
    const search = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Search',
    );
    search?.click();
    expect(store.search).toHaveBeenCalledWith('board');
  });

  it('deletes a note from the drawer', async () => {
    const store = fakeStore();
    await TestBed.configureTestingModule({
      imports: [Memory],
      providers: [provideRouter([]), { provide: MemoryStore, useValue: store }],
    }).compileComponents();
    const fixture = TestBed.createComponent(Memory);
    fixture.detectChanges();
    const element = fixture.nativeElement as HTMLElement;
    (element.querySelector('[data-testid="memory-note"]') as HTMLElement).click();
    fixture.detectChanges();
    const remove = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Delete',
    );
    remove?.click();
    fixture.detectChanges();
    const confirm = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Confirm',
    );
    confirm?.click();
    expect(store.remove).toHaveBeenCalled();
  });
});
