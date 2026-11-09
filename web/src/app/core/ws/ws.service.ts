import { inject, Injectable, OnDestroy, signal } from '@angular/core';
import { RUNTIME_CONFIG } from '../config/runtime-config';
import { ApiClient } from '../api/api-client';
import type { EventEnvelope } from '../../domain/models';

/** Connection state exposed to the health pill. */
export type ConnectionState = 'connected' | 'reconnecting' | 'offline';

type ServerMessage =
  | { readonly type: 'hello'; readonly seq: number }
  | { readonly type: 'ack'; readonly topics: readonly string[] }
  | { readonly type: 'resync'; readonly after: number }
  | { readonly type: 'pong'; readonly ts: number }
  | { readonly type: 'error'; readonly code: string; readonly detail: string }
  | ({
      readonly type: 'event';
    } & EventEnvelope);

const MAX_BACKOFF_MS = 30_000;
const SPEECH_COALESCE_MS = 2_000;

/** Single WebSocket connection with topics, resume and dedupe. */
@Injectable({ providedIn: 'root' })
export class WsService implements OnDestroy {
  private readonly config = inject(RUNTIME_CONFIG);
  private readonly api = inject(ApiClient);
  private socket: WebSocket | null = null;
  private readonly topics = new Map<string, number>();
  private readonly seen = new Set<string>();
  private readonly speech = new Map<string, number>();
  private labId = 'default';
  private lastSeq = 0;
  private backoff = 1_000;
  private heartbeat: ReturnType<typeof setInterval> | null = null;
  private reconnect: ReturnType<typeof setTimeout> | null = null;
  private closedByUser = false;

  readonly state = signal<ConnectionState>('offline');
  readonly lastEvent = signal<EventEnvelope | null>(null);

  connect(labId: string): void {
    this.labId = labId;
    this.closedByUser = false;
    this.open();
  }

  private open(): void {
    if (typeof WebSocket === 'undefined' || this.socket !== null) {
      return;
    }
    const base = this.config.wsBaseUrl.replace(/\/$/, '');
    const url = `${base}/api/v1/ws?lab_id=${encodeURIComponent(this.labId)}&since=${this.lastSeq}`;
    const socket = new WebSocket(url);
    this.socket = socket;
    this.state.set('reconnecting');
    socket.addEventListener('open', () => {
      this.state.set('connected');
      this.backoff = 1_000;
      for (const topic of this.topics.keys()) {
        this.send({ type: 'subscribe', topics: [topic] });
      }
      this.startHeartbeat();
    });
    socket.addEventListener('message', (event: MessageEvent<string>) => {
      this.handle(event.data);
    });
    socket.addEventListener('close', () => {
      this.stopHeartbeat();
      this.socket = null;
      if (this.closedByUser) {
        this.state.set('offline');
        return;
      }
      this.state.set('reconnecting');
      this.scheduleReconnect();
    });
    socket.addEventListener('error', () => {
      socket.close();
    });
  }

  private scheduleReconnect(): void {
    if (this.reconnect !== null) {
      return;
    }
    const jitter = Math.floor(Math.random() * 250);
    this.reconnect = setTimeout(() => {
      this.reconnect = null;
      this.open();
    }, this.backoff + jitter);
    this.backoff = Math.min(this.backoff * 2, MAX_BACKOFF_MS);
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    this.heartbeat = setInterval(() => {
      this.send({ type: 'ping', ts: Date.now() });
    }, 20_000);
  }

  private stopHeartbeat(): void {
    if (this.heartbeat !== null) {
      clearInterval(this.heartbeat);
      this.heartbeat = null;
    }
  }

  private send(message: Record<string, unknown>): void {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(message));
    }
  }

  private handle(raw: string): void {
    let message: ServerMessage;
    try {
      message = JSON.parse(raw) as ServerMessage;
    } catch {
      return;
    }
    if (message.type === 'hello') {
      this.lastSeq = Math.max(this.lastSeq, message.seq);
      return;
    }
    if (message.type === 'resync') {
      this.lastSeq = Math.max(this.lastSeq, message.after);
      void this.backfill();
      return;
    }
    if (message.type !== 'event') {
      return;
    }
    this.apply(message as EventEnvelope);
  }

  private apply(event: EventEnvelope): void {
    if (event.seq <= this.lastSeq || this.seen.has(event.event_id)) {
      return;
    }
    if (event.event === 'office.agent.speech') {
      const agent = String(event.payload['agent_id'] ?? 'unknown');
      const now = Date.now();
      const previous = this.speech.get(agent) ?? 0;
      if (now - previous < SPEECH_COALESCE_MS) {
        return;
      }
      this.speech.set(agent, now);
    }
    this.lastSeq = event.seq;
    this.seen.add(event.event_id);
    if (this.seen.size > 2_000) {
      this.seen.clear();
    }
    this.lastEvent.set(event);
  }

  private async backfill(): Promise<void> {
    try {
      const page = await this.api.listEvents(this.labId, this.lastSeq, 1_000);
      for (const event of page.data) {
        this.apply(event);
      }
    } catch {
      this.scheduleReconnect();
    }
  }

  /** Subscribe to a topic; returns the reference-counted unsubscribe. */
  subscribe(topic: string): () => void {
    const count = this.topics.get(topic) ?? 0;
    this.topics.set(topic, count + 1);
    if (count === 0) {
      this.send({ type: 'subscribe', topics: [topic] });
    }
    let released = false;
    return () => {
      if (released) {
        return;
      }
      released = true;
      const remaining = (this.topics.get(topic) ?? 1) - 1;
      if (remaining <= 0) {
        this.topics.delete(topic);
        this.send({ type: 'unsubscribe', topics: [topic] });
      } else {
        this.topics.set(topic, remaining);
      }
    };
  }

  ngOnDestroy(): void {
    this.closedByUser = true;
    this.stopHeartbeat();
    if (this.reconnect !== null) {
      clearTimeout(this.reconnect);
      this.reconnect = null;
    }
    this.socket?.close();
    this.socket = null;
  }
}
