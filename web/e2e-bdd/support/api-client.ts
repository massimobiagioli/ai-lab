import { randomUUID } from 'node:crypto';
import { env, urlFor } from './env';
import { redact } from './redact';

export type HttpMethod = 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE';

export interface ProblemViolation {
  readonly rule?: string;
  readonly field?: string;
  readonly message?: string;
}

export interface ProblemDetails {
  readonly type?: string;
  readonly title?: string;
  readonly status?: number;
  readonly detail?: string;
  readonly instance?: string;
  readonly code?: string;
  readonly violations?: readonly ProblemViolation[];
  readonly trace_id?: string;
}

export interface ApiResponse<T = unknown> {
  readonly status: number;
  readonly headers: Headers;
  readonly body: T;
  readonly problem?: ProblemDetails;
  readonly requestId: string;
  readonly url: string;
  readonly durationMs: number;
}

export interface RequestOptions {
  readonly body?: unknown;
  readonly headers?: Readonly<Record<string, string>>;
  readonly idempotencyKey?: string;
  readonly timeoutMs?: number;
  readonly retries?: number;
}

function isProblem(value: unknown): value is ProblemDetails {
  return (
    Boolean(value) &&
    typeof value === 'object' &&
    ('code' in (value as Record<string, unknown>) || 'type' in (value as Record<string, unknown>))
  );
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export class ApiClient {
  constructor(
    private readonly baseURL: string = env.baseURL,
    private readonly token: string = env.apiToken,
  ) {}

  async request<T = unknown>(
    method: HttpMethod,
    path: string,
    options: RequestOptions = {},
  ): Promise<ApiResponse<T>> {
    const url = /^https?:\/\//.test(path) ? path : urlFor(path);
    const requestId = randomUUID();
    const retries = options.retries ?? (method === 'GET' ? 2 : 0);
    let lastError: unknown;

    for (let attempt = 0; attempt <= retries; attempt += 1) {
      const started = Date.now();
      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), options.timeoutMs ?? env.timeoutMs);
      try {
        const headers: Record<string, string> = {
          accept: 'application/json',
          'x-request-id': requestId,
          ...(options.headers ?? {}),
        };
        if (this.token) headers['authorization'] = `Bearer ${this.token}`;
        if (options.idempotencyKey) headers['idempotency-key'] = options.idempotencyKey;
        let body: string | undefined;
        if (options.body !== undefined) {
          headers['content-type'] = 'application/json';
          body = JSON.stringify(options.body);
        }

        const init: RequestInit = { method, headers, signal: controller.signal };
        if (body !== undefined) init.body = body;
        const response = await fetch(url, init);
        const text = await response.text();
        let parsed: unknown = text;
        if (text.length > 0 && (response.headers.get('content-type') ?? '').includes('json')) {
          try {
            parsed = JSON.parse(text) as unknown;
          } catch {
            parsed = text;
          }
        }
        const durationMs = Date.now() - started;
        if ([502, 503, 504].includes(response.status) && attempt < retries) {
          await delay(100 * (attempt + 1) + Math.floor(Math.random() * 100));
          continue;
        }
        return {
          status: response.status,
          headers: response.headers,
          body: parsed as T,
          ...(isProblem(parsed) ? { problem: parsed } : {}),
          requestId: response.headers.get('x-request-id') ?? requestId,
          url,
          durationMs,
        };
      } catch (error) {
        lastError = error;
        if (attempt < retries) {
          await delay(100 * (attempt + 1) + Math.floor(Math.random() * 100));
          continue;
        }
      } finally {
        clearTimeout(timeout);
      }
    }
    throw new Error(`API request failed: ${method} ${url}: ${redact(String(lastError))}`);
  }

  get<T = unknown>(path: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    return this.request<T>('GET', path, options);
  }

  post<T = unknown>(path: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    return this.request<T>('POST', path, options);
  }

  patch<T = unknown>(path: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    return this.request<T>('PATCH', path, options);
  }

  delete<T = unknown>(path: string, options: RequestOptions = {}): Promise<ApiResponse<T>> {
    return this.request<T>('DELETE', path, options);
  }

  health(): Promise<ApiResponse> {
    return this.get('/health/live');
  }

  ready(): Promise<ApiResponse> {
    return this.get('/health/ready');
  }
}
