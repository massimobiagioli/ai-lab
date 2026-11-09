import { computed, signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { DEMO_CARDS } from '../../core/demo/demo-data';
import { WsService } from '../../core/ws/ws.service';
import { Board } from './board';
import { BoardStore } from './store/board.store';

function fakeStore() {
  const cards = signal(DEMO_CARDS);
  return {
    state: signal('ready'),
    cards,
    error: signal(null),
    lastViolation: signal(null),
    demo: signal(true),
    clearViolation: vi.fn(),
    columns: computed(() => {
      const groups = new Map<string, (typeof DEMO_CARDS)[number][]>();
      for (const card of cards()) {
        const list = groups.get(card.column) ?? [];
        list.push(card);
        groups.set(card.column, list);
      }
      return groups;
    }),
    load: vi.fn(),
    moveCard: vi.fn(),
  };
}

describe('Board', () => {
  it('renders the six columns and cards', async () => {
    await TestBed.configureTestingModule({
      imports: [Board],
      providers: [
        provideRouter([]),
        { provide: BoardStore, useValue: fakeStore() },
        { provide: WsService, useValue: { lastEvent: signal(null) } },
      ],
    }).compileComponents();
    const fixture = TestBed.createComponent(Board);
    fixture.componentRef.setInput('projectId', 'demo');
    fixture.detectChanges();
    const element = fixture.nativeElement as HTMLElement;
    expect(element.querySelectorAll('[data-testid^="column-"]').length).toBe(6);
    expect(element.querySelectorAll('[data-testid^="card-"]').length).toBe(5);
  });

  it('moves a card with the keyboard', async () => {
    const store = fakeStore();
    await TestBed.configureTestingModule({
      imports: [Board],
      providers: [
        provideRouter([]),
        { provide: BoardStore, useValue: store },
        { provide: WsService, useValue: { lastEvent: signal(null) } },
      ],
    }).compileComponents();
    const fixture = TestBed.createComponent(Board);
    fixture.componentRef.setInput('projectId', 'demo');
    fixture.detectChanges();
    const card = (fixture.nativeElement as HTMLElement).querySelector<HTMLElement>(
      '[data-testid="card-AIL-2"]',
    );
    card?.dispatchEvent(new KeyboardEvent('keydown', { key: 'm' }));
    card?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
    card?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }));
    expect(store.moveCard).toHaveBeenCalledWith('card-2', 'DOING');
  });

  it('opens the card detail drawer', async () => {
    await TestBed.configureTestingModule({
      imports: [Board],
      providers: [
        provideRouter([]),
        { provide: BoardStore, useValue: fakeStore() },
        { provide: WsService, useValue: { lastEvent: signal(null) } },
      ],
    }).compileComponents();
    const fixture = TestBed.createComponent(Board);
    fixture.componentRef.setInput('projectId', 'demo');
    fixture.detectChanges();
    const card = (fixture.nativeElement as HTMLElement).querySelector<HTMLElement>(
      '[data-testid="card-AIL-1"]',
    );
    card?.click();
    fixture.detectChanges();
    expect((fixture.nativeElement as HTMLElement).textContent).toContain('Build the Kanban board');
  });
});
