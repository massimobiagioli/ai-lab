export interface BddEnvironment {
  readonly baseURL: string;
  readonly apiToken: string;
  readonly live: boolean;
  readonly timeoutMs: number;
  readonly pollIntervalMs: number;
  readonly viewport: { readonly width: number; readonly height: number };
  readonly visualThreshold: number;
  readonly reducedMotion: 'reduce' | 'no-preference';
  readonly reportsDir: string;
  readonly screenshotsDir: string;
  readonly tracesDir: string;
  readonly videosDir: string;
}

function numberFromEnv(name: string, fallback: number): number {
  const value = process.env[name];
  if (!value) return fallback;
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

export const env: BddEnvironment = {
  baseURL: process.env['AILAB_BDD_BASE_URL'] ?? 'http://127.0.0.1:8080',
  apiToken: process.env['AILAB_API_TOKEN'] ?? '',
  live: process.env['AILAB_BDD_LIVE'] === '1',
  timeoutMs: numberFromEnv('AILAB_BDD_TIMEOUT_MS', 15_000),
  pollIntervalMs: numberFromEnv('AILAB_BDD_POLL_MS', 100),
  viewport: {
    width: numberFromEnv('AILAB_BDD_VIEWPORT_WIDTH', 1440),
    height: numberFromEnv('AILAB_BDD_VIEWPORT_HEIGHT', 900),
  },
  visualThreshold: 0.001,
  reducedMotion: process.env['AILAB_BDD_REDUCED_MOTION'] === 'reduce' ? 'reduce' : 'no-preference',
  reportsDir: 'reports',
  screenshotsDir: 'reports/screenshots',
  tracesDir: 'reports/test-results',
  videosDir: 'reports/videos',
};

export function urlFor(path: string): string {
  if (/^https?:\/\//.test(path)) return path;
  return `${env.baseURL.replace(/\/+$/, '')}/${path.replace(/^\/+/, '')}`;
}
