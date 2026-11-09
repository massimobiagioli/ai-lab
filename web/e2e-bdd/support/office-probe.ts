import type { Page } from '@playwright/test';

export const AMIGA_PALETTE: readonly string[] = [
  '#0b1026',
  '#141b3c',
  '#2e3a6b',
  '#5b6fb0',
  '#9fb3e6',
  '#f2e9d8',
  '#f2b84b',
  '#e07a3f',
  '#c0392b',
  '#2a9d8f',
  '#57c785',
  '#c45aa0',
  '#000000',
  '#ffffff',
  '#1b1b1b',
  '#c9c9c9',
  '#e8b98f',
  '#8d5524',
  '#5c3317',
  '#c68642',
  '#f1c27d',
  '#3b1f14',
  '#7a5230',
  '#2b1d16',
  '#141414',
  '#b0b0b0',
  '#6b4f3a',
  '#3b2a1a',
  '#1a1a1a',
  '#2a9d8f',
  '#57c785',
  '#e07a3f',
];

export interface CanvasProbe {
  readonly found: boolean;
  readonly bufferWidth: number;
  readonly bufferHeight: number;
  readonly cssWidth: number;
  readonly cssHeight: number;
  readonly scale: number;
  readonly distinctColors: readonly string[];
  readonly offPalette: readonly string[];
  readonly scanlineOpacity: number;
  readonly frameHash?: string;
  readonly drawCalls?: number;
  readonly error?: string;
}

declare global {
  interface Window {
    __ailabOffice?: {
      frameHash?: () => string;
      drawCalls?: () => number;
    };
  }
}

function hex(value: number): string {
  return value.toString(16).padStart(2, '0');
}

export async function probeOffice(page: Page): Promise<CanvasProbe> {
  return page.evaluate((palette) => {
    const canvas = document.querySelector<HTMLCanvasElement>(
      '[data-testid="office-canvas"], canvas.office-canvas, canvas',
    );
    if (!canvas) {
      return {
        found: false,
        bufferWidth: 0,
        bufferHeight: 0,
        cssWidth: 0,
        cssHeight: 0,
        scale: 0,
        distinctColors: [],
        offPalette: [],
        scanlineOpacity: 0,
      };
    }
    const rect = canvas.getBoundingClientRect();
    const scale = Math.min(rect.width / 384, rect.height / 216);
    const context = canvas.getContext('2d', { willReadFrequently: true });
    let distinctColors: string[] = [];
    let offPalette: string[] = [];
    let error: string | undefined;
    try {
      if (context) {
        const data = context.getImageData(0, 0, canvas.width, canvas.height).data;
        const seen = new Set<string>();
        for (let i = 0; i < data.length; i += 4) {
          const alpha = data[i + 3] ?? 0;
          if (alpha === 0) continue;
          const color = `#${(data[i] ?? 0).toString(16).padStart(2, '0')}${(data[i + 1] ?? 0).toString(16).padStart(2, '0')}${(data[i + 2] ?? 0).toString(16).padStart(2, '0')}`;
          seen.add(color);
        }
        distinctColors = [...seen].sort();
        const allowed = new Set(palette);
        offPalette = distinctColors.filter((color) => !allowed.has(color));
      }
    } catch (caught) {
      error = caught instanceof Error ? caught.message : String(caught);
    }
    const scanline = document.querySelector<HTMLElement>(
      '[data-testid="office-scanlines"], .scanlines',
    );
    const scanlineOpacity = scanline ? Number(getComputedStyle(scanline).opacity) : 0;
    const hook = window.__ailabOffice;
    const frameHash = hook?.frameHash?.();
    const drawCalls = hook?.drawCalls?.();
    return {
      found: true,
      bufferWidth: canvas.width,
      bufferHeight: canvas.height,
      cssWidth: rect.width,
      cssHeight: rect.height,
      scale,
      distinctColors,
      offPalette,
      scanlineOpacity,
      ...(frameHash !== undefined ? { frameHash } : {}),
      ...(drawCalls !== undefined ? { drawCalls } : {}),
      ...(error ? { error } : {}),
    };
  }, AMIGA_PALETTE);
}

export async function measureFps(
  page: Page,
  durationMs = 2000,
): Promise<{ average: number; p95: number; samples: number }> {
  const samples = await page.evaluate(
    (duration) =>
      new Promise<number[]>((resolve) => {
        const frames: number[] = [];
        const start = performance.now();
        let previous = start;
        const tick = (now: number): void => {
          frames.push(now - previous);
          previous = now;
          if (now - start >= duration) resolve(frames);
          else requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
      }),
    durationMs,
  );
  const fps = samples
    .filter((ms) => ms > 0)
    .map((ms) => 1000 / ms)
    .sort((a, b) => a - b);
  const average = fps.reduce((sum, value) => sum + value, 0) / Math.max(fps.length, 1);
  const p95 = fps[Math.min(fps.length - 1, Math.floor(fps.length * 0.95))] ?? 0;
  return { average, p95, samples: fps.length };
}

export async function measureHeapMB(page: Page): Promise<number | undefined> {
  return page.evaluate(() => {
    const legacy = performance as Performance & { memory?: { usedJSHeapSize: number } };
    return legacy.memory ? legacy.memory.usedJSHeapSize / (1024 * 1024) : undefined;
  });
}

export function normalizeHex(value: string): string {
  return value.startsWith('#') ? value.toLowerCase() : value;
}

export function formatColor(red: number, green: number, blue: number): string {
  return `#${hex(red)}${hex(green)}${hex(blue)}`;
}
