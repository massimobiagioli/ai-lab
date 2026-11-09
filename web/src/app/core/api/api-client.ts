import { inject, Injectable } from '@angular/core';
import { RUNTIME_CONFIG } from '../config/runtime-config';
import { sessionToken } from '../auth/session';
import type { Page, Problem, RunView, StepView } from '../../domain/models';
import { ApiProblem, toProblem } from './problem';

interface RequestOptions {
  readonly method?: 'GET' | 'POST' | 'PATCH' | 'DELETE';
  readonly body?: unknown;
  readonly signal?: AbortSignal;
  readonly retry?: boolean;
  readonly idempotencyKey?: string;
}

const MAX_RETRIES = 2;
const TIMEOUT_MS = 10_000;

/** Typed edge client with correlation, timeout, retry and Problem normalization. */
@Injectable({ providedIn: 'root' })
export class ApiClient {
  private readonly config = inject(RUNTIME_CONFIG);

  private url(path: string): string {
    return `${this.config.apiBaseUrl.replace(/\/$/, '')}${path}`;
  }

  private headers(): HeadersInit {
    const token = sessionToken.read() ?? this.config.token;
    const headers: Record<string, string> = {
      accept: 'application/json',
      'x-request-id': crypto.randomUUID(),
    };
    if (token !== null) {
      headers['authorization'] = `Bearer ${token}`;
    }
    return headers;
  }

  private async send<T>(path: string, options: RequestOptions = {}): Promise<T> {
    const method = options.method ?? 'GET';
    const retryable = options.retry ?? method === 'GET';
    const headers = this.headers() as Record<string, string>;
    if (options.idempotencyKey !== undefined) {
      headers['idempotency-key'] = options.idempotencyKey;
    }
    let lastError: unknown = null;
    for (let attempt = 0; attempt <= (retryable ? MAX_RETRIES : 0); attempt += 1) {
      if (attempt > 0) {
        const jitter = Math.floor(Math.random() * 100);
        await new Promise((resolve) => setTimeout(resolve, 150 * 2 ** (attempt - 1) + jitter));
      }
      try {
        const controller = new AbortController();
        const timeout = setTimeout(() => controller.abort(), TIMEOUT_MS);
        const signal = options.signal
          ? AbortSignal.any([options.signal, controller.signal])
          : controller.signal;
        try {
          const init: RequestInit = { method, headers, signal };
          if (options.body !== undefined) {
            headers['content-type'] = 'application/json';
            init.body = JSON.stringify(options.body);
          }
          const response = await fetch(this.url(path), init);
          if (response.ok) {
            return (await response.json()) as T;
          }
          const body = (await response.json().catch(() => null)) as Partial<{
            code: string;
            title: string;
            detail: string;
          }> | null;
          const problem: Problem = {
            code: body?.code ?? `HTTP_${response.status}`,
            title: body?.title ?? response.statusText,
            status: response.status,
            ...(body?.detail !== undefined ? { detail: body.detail } : {}),
          };
          throw new ApiProblem(problem);
        } finally {
          clearTimeout(timeout);
        }
      } catch (error) {
        lastError = error;
        if (error instanceof ApiProblem && (error.status ?? 500) < 500) {
          throw error;
        }
      }
    }
    throw new ApiProblem(toProblem(lastError));
  }

  listRuns(labId: string, cursor?: string): Promise<Page<RunView>> {
    const query = cursor === undefined ? '' : `?cursor=${encodeURIComponent(cursor)}`;
    return this.send<Page<RunView>>(`/labs/${encodeURIComponent(labId)}/runs${query}`);
  }

  getRun(runId: string): Promise<RunView> {
    return this.send<RunView>(`/runs/${encodeURIComponent(runId)}`);
  }

  getSteps(runId: string, cursor?: string): Promise<Page<StepView>> {
    const query = cursor === undefined ? '' : `?cursor=${encodeURIComponent(cursor)}`;
    return this.send<Page<StepView>>(`/runs/${encodeURIComponent(runId)}/steps${query}`);
  }

  cancelRun(runId: string): Promise<RunView> {
    return this.send<RunView>(`/runs/${encodeURIComponent(runId)}/cancel`, { method: 'POST' });
  }

  enqueueRun(cardId: string, idempotencyKey: string): Promise<RunView> {
    return this.send<RunView>(`/cards/${encodeURIComponent(cardId)}/runs`, {
      method: 'POST',
      retry: false,
      idempotencyKey,
    });
  }
}
