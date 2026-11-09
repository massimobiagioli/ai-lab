import type { TestInfo } from '@playwright/test';
import type { ApiResponse } from './api-client';

export interface StoredValue {
  readonly value: unknown;
  readonly at: number;
}

export class World {
  readonly scenarioId: string;
  readonly scenarioTitle: string;
  readonly tags: readonly string[];
  readonly startedAt: number;
  readonly values = new Map<string, StoredValue>();
  readonly artifacts: string[] = [];
  lastResponse?: ApiResponse;

  constructor(testInfo: TestInfo) {
    this.scenarioTitle = testInfo.title;
    this.scenarioId = /E2E-\d{3}/.exec(testInfo.title)?.[0] ?? 'E2E-UNKNOWN';
    this.tags = testInfo.tags;
    this.startedAt = Date.now();
  }

  set(key: string, value: unknown): void {
    this.values.set(key, { value, at: Date.now() });
  }

  get<T = unknown>(key: string): T | undefined {
    return this.values.get(key)?.value as T | undefined;
  }

  require<T = unknown>(key: string): T {
    const stored = this.values.get(key);
    if (!stored) throw new Error(`${this.scenarioId}: fixture "${key}" is not set`);
    return stored.value as T;
  }

  recordArtifact(path: string): void {
    if (!this.artifacts.includes(path)) this.artifacts.push(path);
  }
}
