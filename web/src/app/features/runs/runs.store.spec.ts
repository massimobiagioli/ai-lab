import { TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { ApiClient } from '../../core/api/api-client';
import type { RunView } from '../../domain/models';
import { RunsStore } from './runs.store';

const run: RunView = {
  id: 'run-1',
  card_id: 'card-1',
  lab_id: 'lab-1',
  state: 'running',
  attempt: 1,
  started_at: '2026-11-07T10:00:00Z',
  ended_at: null,
  cost_microusd: 0,
  created_at: '2026-11-07T10:00:00Z',
};

describe('RunsStore', () => {
  const api = {
    listRuns: vi.fn(),
    cancelRun: vi.fn(),
  };

  beforeEach(() => {
    api.listRuns.mockReset();
    api.cancelRun.mockReset();
    TestBed.configureTestingModule({
      providers: [{ provide: ApiClient, useValue: api }],
    });
  });

  it('loads runs into ready state', async () => {
    api.listRuns.mockResolvedValue({ data: [run], next_cursor: null, total_estimate: 1 });
    const store = TestBed.inject(RunsStore);
    await store.load('lab-1');
    expect(store.state()).toBe('ready');
    expect(store.runs().length).toBe(1);
    expect(store.counters()).toEqual({ total: 1, running: 1, succeeded: 0, failed: 0 });
  });

  it('reports errors', async () => {
    api.listRuns.mockRejectedValue(new Error('offline'));
    const store = TestBed.inject(RunsStore);
    await store.load();
    expect(store.state()).toBe('error');
    expect(store.error()?.code).toBe('NETWORK_ERROR');
  });

  it('patches a cancelled run', async () => {
    api.listRuns.mockResolvedValue({ data: [run], next_cursor: null, total_estimate: 1 });
    api.cancelRun.mockResolvedValue({ ...run, state: 'cancelled' });
    const store = TestBed.inject(RunsStore);
    await store.load();
    await store.cancel('run-1');
    expect(store.runs()[0]?.state).toBe('cancelled');
  });
});
