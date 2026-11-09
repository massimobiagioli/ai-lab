import { InjectionToken } from '@angular/core';

/** Runtime configuration injected at boot. */
export interface RuntimeConfig {
  readonly apiBaseUrl: string;
  readonly wsBaseUrl: string;
  readonly token: string | null;
}

declare global {
  interface Window {
    __AILAB_CONFIG__?: Partial<RuntimeConfig>;
  }
}

/** Resolve configuration from `window.__AILAB_CONFIG__` with local defaults. */
export function resolveRuntimeConfig(): RuntimeConfig {
  const injected = window.__AILAB_CONFIG__ ?? {};
  const base = injected.apiBaseUrl ?? '/api/v1';
  return {
    apiBaseUrl: base,
    wsBaseUrl:
      injected.wsBaseUrl ?? `${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}`,
    token: injected.token ?? null,
  };
}

/** Runtime configuration token. */
export const RUNTIME_CONFIG = new InjectionToken<RuntimeConfig>('RUNTIME_CONFIG', {
  providedIn: 'root',
  factory: resolveRuntimeConfig,
});
