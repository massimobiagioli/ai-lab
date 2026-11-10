import { inject, Injectable } from '@angular/core';
import { RUNTIME_CONFIG } from '../config/runtime-config';
import { sessionToken } from '../auth/session';
import type {
  Agent,
  Card,
  EventEnvelope,
  IdeaAck,
  LabSettings,
  LabSummary,
  MemoryNote,
  Page,
  Problem,
  Project,
  Provider,
  RunView,
  Sow,
  StepView,
} from '../../domain/models';
import {
  DEMO_AGENTS,
  DEMO_CARDS,
  DEMO_EVENTS,
  DEMO_LABS,
  DEMO_LAB_ID,
  DEMO_PROJECTS,
  DEMO_PROVIDERS,
  DEMO_RUNS,
  DEMO_SETTINGS,
  DEMO_SOW,
  DEMO_MEMORY,
  isDemoFallback,
} from '../demo/demo-data';
import { signal } from '@angular/core';
import { createId } from '../ids';
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
  /** True while the last read used the explicit demo read model. */
  readonly usingDemo = signal(false);

  private isDemoUnavailable(error: unknown): boolean {
    return (
      isDemoFallback(error) ||
      (error instanceof ApiProblem &&
        (error.status === 401 || error.status === 403 || error.status === 422))
    );
  }

  private async withDemo<T>(path: string, demo: T): Promise<T> {
    try {
      const value = await this.send<T>(path);
      this.usingDemo.set(false);
      return value;
    } catch (error) {
      if (this.isDemoUnavailable(error)) {
        this.usingDemo.set(true);
        return demo;
      }
      throw error;
    }
  }

  private url(path: string): string {
    return `${this.config.apiBaseUrl.replace(/\/$/, '')}${path}`;
  }

  private headers(): HeadersInit {
    const token = sessionToken.read() ?? this.config.token;
    const headers: Record<string, string> = {
      accept: 'application/json',
      'x-request-id': createId(),
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

  private static unwrapList<T>(
    value: readonly T[] | { readonly data: readonly T[] },
  ): readonly T[] {
    return 'data' in value ? value.data : value;
  }

  getBoard(projectId: string): Promise<readonly Card[]> {
    return this.withDemo(`/projects/${encodeURIComponent(projectId)}/board`, DEMO_CARDS);
  }

  async listLabs(): Promise<readonly LabSummary[]> {
    const value = await this.withDemo<
      readonly LabSummary[] | { readonly data: readonly LabSummary[] }
    >('/labs', DEMO_LABS);
    return ApiClient.unwrapList(value);
  }

  async listAgents(): Promise<readonly Agent[]> {
    const value = await this.withDemo<readonly Agent[] | { readonly data: readonly Agent[] }>(
      '/agents',
      DEMO_AGENTS,
    );
    return ApiClient.unwrapList(value);
  }

  async listProjects(): Promise<readonly Project[]> {
    const value = await this.withDemo<readonly Project[] | { readonly data: readonly Project[] }>(
      '/projects',
      DEMO_PROJECTS,
    );
    return ApiClient.unwrapList(value);
  }

  async listProviders(): Promise<readonly Provider[]> {
    const value = await this.withDemo<readonly Provider[] | { readonly data: readonly Provider[] }>(
      '/providers',
      DEMO_PROVIDERS,
    );
    return ApiClient.unwrapList(value);
  }

  getSow(projectId: string): Promise<Sow> {
    return this.withDemo(`/sows?project_id=${encodeURIComponent(projectId)}`, DEMO_SOW);
  }

  searchMemory(query: string): Promise<readonly MemoryNote[]> {
    return this.withDemo(
      `/memory/search?q=${encodeURIComponent(query)}`,
      DEMO_MEMORY.filter(
        (note) => note.title.toLowerCase().includes(query.toLowerCase()) || query === '',
      ),
    );
  }

  getSettings(labId: string): Promise<LabSettings> {
    return this.withDemo(`/settings?lab_id=${encodeURIComponent(labId)}`, DEMO_SETTINGS);
  }

  async setupRequired(): Promise<boolean> {
    try {
      const [labs, providers, projects, agents] = await Promise.all([
        this.send<{ readonly data: readonly unknown[] }>('/labs'),
        this.send<{ readonly data: readonly unknown[] }>('/providers'),
        this.send<{ readonly data: readonly unknown[] }>('/projects'),
        this.send<{ readonly data: readonly unknown[] }>('/agents'),
      ]);
      return (
        labs.data.length === 0 ||
        providers.data.length === 0 ||
        projects.data.length === 0 ||
        agents.data.length === 0
      );
    } catch {
      return false;
    }
  }

  createProvider(input: {
    readonly name: string;
    readonly kind: 'deepseek';
    readonly base_url: string;
    readonly api_key: string;
  }): Promise<Provider> {
    return this.send<Provider>('/providers', { method: 'POST', body: input, retry: false });
  }

  createLab(input: {
    readonly name: string;
    readonly slug: string;
    readonly description?: string;
    readonly autonomy_level?: number;
    readonly monthly_budget_microusd?: number;
    readonly tech_lead: {
      readonly name: string;
      readonly role: 'tech_lead';
      readonly character_id: string;
      readonly description: string;
    };
  }): Promise<{ readonly lab: { readonly id: string; readonly name: string } }> {
    return this.send('/labs', { method: 'POST', body: input, retry: false });
  }

  createProject(input: {
    readonly lab_id: string;
    readonly name: string;
    readonly slug: string;
    readonly key_prefix: string;
    readonly description?: string;
    readonly source_kind?: 'github';
    readonly source_ref: string;
    readonly default_branch?: string;
  }): Promise<{ readonly project: { readonly id: string } }> {
    return this.send('/projects', { method: 'POST', body: input, retry: false });
  }

  transitionCard(cardId: string, to: string): Promise<Card> {
    return this.send<Card>(`/cards/${encodeURIComponent(cardId)}/transition`, {
      method: 'POST',
      body: { to },
      retry: false,
    });
  }

  async listRuns(labId: string = DEMO_LAB_ID, cursor?: string): Promise<Page<RunView>> {
    const query = cursor === undefined ? '' : `?cursor=${encodeURIComponent(cursor)}`;
    try {
      const page = await this.send<Page<RunView>>(
        `/labs/${encodeURIComponent(labId)}/runs${query}`,
      );
      this.usingDemo.set(false);
      return page;
    } catch (error) {
      if (this.isDemoUnavailable(error)) {
        this.usingDemo.set(true);
        return { data: DEMO_RUNS, next_cursor: null, total_estimate: DEMO_RUNS.length };
      }
      throw error;
    }
  }

  async listEvents(labId: string, after: number, limit = 1_000): Promise<Page<EventEnvelope>> {
    try {
      const page = await this.send<Page<EventEnvelope>>(
        `/events?lab_id=${encodeURIComponent(labId)}&after=${after}&limit=${limit}`,
      );
      this.usingDemo.set(false);
      return page;
    } catch (error) {
      if (this.isDemoUnavailable(error)) {
        this.usingDemo.set(true);
        return {
          data: DEMO_EVENTS,
          next_cursor: null,
          total_estimate: DEMO_EVENTS.length,
        };
      }
      throw error;
    }
  }

  async submitIdea(text: string): Promise<IdeaAck> {
    try {
      const ack = await this.send<IdeaAck>('/ideas', {
        method: 'POST',
        body: { text },
        retry: false,
      });
      this.usingDemo.set(false);
      return ack;
    } catch (error) {
      if (this.isDemoUnavailable(error)) {
        this.usingDemo.set(true);
        return { id: `idea-${Date.now()}`, status: 'queued' };
      }
      throw error;
    }
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
