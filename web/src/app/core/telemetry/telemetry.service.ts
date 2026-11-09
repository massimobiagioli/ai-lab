import { inject, Injectable } from '@angular/core';
import { RUNTIME_CONFIG } from '../config/runtime-config';

type TelemetryLevel = 'debug' | 'info' | 'warn' | 'error';

interface TelemetryPayload {
  readonly level: TelemetryLevel;
  readonly message: string;
  readonly route: string;
  readonly build: string;
  stack?: string;
  trace_id?: string;
}

/** Fire-and-forget frontend telemetry. Never throws into the caller. */
@Injectable({ providedIn: 'root' })
export class Telemetry {
  private readonly config = inject(RUNTIME_CONFIG);

  capture(level: TelemetryLevel, message: string, error?: unknown): void {
    const payload: TelemetryPayload = {
      level,
      message: message.slice(0, 1_024),
      route: location.pathname,
      build: 'dev',
    };
    if (error instanceof Error && error.stack !== undefined) {
      payload.stack = error.stack.slice(0, 4_096);
    }
    void fetch(`${this.config.apiBaseUrl.replace(/\/$/, '')}/telemetry/frontend`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(payload),
      keepalive: true,
    }).catch(() => undefined);
  }
}
