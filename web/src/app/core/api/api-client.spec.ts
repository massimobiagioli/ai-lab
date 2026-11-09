import { TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { RUNTIME_CONFIG } from '../config/runtime-config';
import { ApiClient } from './api-client';
import { ApiProblem } from './problem';

const config = { apiBaseUrl: '/api/v1', wsBaseUrl: 'ws://localhost', token: null };

describe('ApiClient', () => {
  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [{ provide: RUNTIME_CONFIG, useValue: config }],
    });
  });

  it('normalizes a problem response', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ code: 'RUN_NOT_ALLOWED', title: 'Not allowed' }), {
        status: 422,
        headers: { 'content-type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);
    const client = TestBed.inject(ApiClient);
    await expect(client.getRun('run-1')).rejects.toBeInstanceOf(ApiProblem);
    vi.unstubAllGlobals();
  });

  it('returns a parsed page', async () => {
    const page = { data: [], next_cursor: null, total_estimate: 0 };
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify(page), {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);
    const client = TestBed.inject(ApiClient);
    await expect(client.listRuns('lab-1')).resolves.toEqual(page);
    vi.unstubAllGlobals();
  });
});
