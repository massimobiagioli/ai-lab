import { TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { ApiClient } from '../api/api-client';
import { RUNTIME_CONFIG } from '../config/runtime-config';
import { WsService } from './ws.service';

class FakeSocket {
  static readonly OPEN = 1;
  static instances: FakeSocket[] = [];
  readyState = FakeSocket.OPEN;
  private readonly listeners = new Map<string, Array<(event: { data?: string }) => void>>();

  constructor(public readonly url: string) {
    FakeSocket.instances.push(this);
  }

  addEventListener(type: string, listener: (event: { data?: string }) => void): void {
    const list = this.listeners.get(type) ?? [];
    list.push(listener);
    this.listeners.set(type, list);
  }

  send(): void {}
  close(): void {}

  emit(type: string, event: { data?: string } = {}): void {
    for (const listener of this.listeners.get(type) ?? []) {
      listener(event);
    }
  }
}

const config = { apiBaseUrl: '/api/v1', wsBaseUrl: 'ws://localhost', token: null };

describe('WsService', () => {
  it('backfills missing events on resync', async () => {
    FakeSocket.instances = [];
    vi.stubGlobal('WebSocket', FakeSocket as unknown as typeof WebSocket);
    const api = {
      listEvents: vi.fn().mockResolvedValue({
        data: [
          {
            event_id: 'evt-1',
            event: 'kanban.card.moved',
            seq: 5,
            lab_id: 'lab-1',
            occurred_at: '2026-11-09T00:00:00Z',
            payload: {},
          },
        ],
        next_cursor: null,
        total_estimate: 1,
      }),
    };
    TestBed.configureTestingModule({
      providers: [
        { provide: RUNTIME_CONFIG, useValue: config },
        { provide: ApiClient, useValue: api },
      ],
    });
    const service = TestBed.inject(WsService);
    service.connect('lab-1');
    const socket = FakeSocket.instances[0];
    expect(socket).toBeDefined();
    socket?.emit('open');
    socket?.emit('message', { data: JSON.stringify({ type: 'resync', after: 0 }) });
    await vi.waitFor(() => expect(service.lastEvent()).not.toBeNull());
    expect(api.listEvents).toHaveBeenCalledWith('lab-1', 0, 1_000);
    expect(service.lastEvent()?.event).toBe('kanban.card.moved');
    vi.unstubAllGlobals();
  });

  it('ignores stale and duplicate events', async () => {
    FakeSocket.instances = [];
    vi.stubGlobal('WebSocket', FakeSocket as unknown as typeof WebSocket);
    const api = { listEvents: vi.fn() };
    TestBed.resetTestingModule();
    TestBed.configureTestingModule({
      providers: [
        { provide: RUNTIME_CONFIG, useValue: config },
        { provide: ApiClient, useValue: api },
      ],
    });
    const service = TestBed.inject(WsService);
    service.connect('lab-1');
    const socket = FakeSocket.instances[0];
    socket?.emit('open');
    const event = {
      event_id: 'evt-1',
      event: 'kanban.card.moved',
      seq: 5,
      lab_id: 'lab-1',
      occurred_at: '2026-11-09T00:00:00Z',
      payload: {},
    };
    socket?.emit('message', { data: JSON.stringify({ type: 'event', ...event }) });
    socket?.emit('message', { data: JSON.stringify({ type: 'event', ...event, seq: 4 }) });
    socket?.emit('message', { data: JSON.stringify({ type: 'event', ...event }) });
    expect(service.lastEvent()?.seq).toBe(5);
    socket?.emit('close');
    expect(service.state()).toBe('reconnecting');
    vi.unstubAllGlobals();
  });
});
