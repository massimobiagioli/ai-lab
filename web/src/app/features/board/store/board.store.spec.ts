import { TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { ApiClient } from '../../../core/api/api-client';
import { ApiProblem } from '../../../core/api/problem';
import { DEMO_CARDS } from '../../../core/demo/demo-data';
import { BoardStore } from './board.store';

describe('BoardStore', () => {
  const api = {
    getBoard: vi.fn(),
    transitionCard: vi.fn(),
    usingDemo: () => false,
  };

  beforeEach(() => {
    api.getBoard.mockReset();
    api.transitionCard.mockReset();
    TestBed.configureTestingModule({
      providers: [{ provide: ApiClient, useValue: api }],
    });
  });

  it('loads the board and groups columns', async () => {
    api.getBoard.mockResolvedValue(DEMO_CARDS);
    const store = TestBed.inject(BoardStore);
    await store.load('demo');
    expect(store.state()).toBe('ready');
    expect(store.cards().length).toBe(5);
    expect(store.columns().get('DOING')?.length).toBe(1);
  });

  it('keeps the optimistic move on the demo fallback', async () => {
    api.getBoard.mockResolvedValue(DEMO_CARDS);
    api.transitionCard.mockRejectedValue(new ApiProblem({ code: 'NOT_IMPLEMENTED', status: 501 }));
    const store = TestBed.inject(BoardStore);
    await store.load('demo');
    await store.moveCard('card-2', 'DOING');
    expect(store.cards().find((card) => card.id === 'card-2')?.column).toBe('DOING');
    expect(store.error()).toBeNull();
  });

  it('rolls back on a 422', async () => {
    api.getBoard.mockResolvedValue(DEMO_CARDS);
    api.transitionCard.mockRejectedValue(
      new ApiProblem({ code: 'INVALID_TRANSITION', status: 422 }),
    );
    const store = TestBed.inject(BoardStore);
    await store.load('demo');
    await store.moveCard('card-2', 'DONE');
    expect(store.cards().find((card) => card.id === 'card-2')?.column).toBe('READY');
    expect(store.error()?.code).toBe('INVALID_TRANSITION');
  });
});
