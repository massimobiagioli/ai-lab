import AxeBuilder from '@axe-core/playwright';
import type { Page } from '@playwright/test';
import { expect } from '@playwright/test';
import type { ApiClient } from './api-client';
import { env } from './env';
import { officeFrameHash } from './frame-hash';
import { measureFps, probeOffice } from './office-probe';
import { firstVisible } from './selectors';
import { dynamicOfficeMasks, expectVisual } from './visual';
import type { World } from './world';

export interface GeneratedStepContext {
  readonly api: ApiClient;
  readonly page: Page;
  readonly world: World;
}

export class UnsupportedStepError extends Error {
  constructor(scenarioId: string, keyword: string, text: string) {
    super(`${scenarioId}: unsupported ${keyword} step: ${text}`);
    this.name = 'UnsupportedStepError';
  }
}

type Handler = (match: RegExpExecArray, context: GeneratedStepContext) => Promise<void> | void;

interface Pattern {
  readonly keyword: 'Given' | 'When' | 'Then';
  readonly regex: RegExp;
  readonly handler: Handler;
}

function readPath(value: unknown, path: string): unknown {
  let current: unknown = value;
  for (const part of path.split('.')) {
    if (current === null || current === undefined) return undefined;
    if (Array.isArray(current)) current = current[Number(part)];
    else if (typeof current === 'object') current = (current as Record<string, unknown>)[part];
    else return undefined;
  }
  return current;
}

function responseOf(context: GeneratedStepContext): {
  status: number;
  body: unknown;
  problem?: { code?: string; violations?: readonly { field?: string; rule?: string }[] };
  headers?: Headers;
} {
  if (!context.world.lastResponse)
    throw new UnsupportedStepError(
      context.world.scenarioId,
      'Then',
      'no API response recorded for this assertion',
    );
  return context.world.lastResponse;
}

function absolute(path: string): string {
  if (/^https?:\/\//.test(path)) return path;
  return `${env.baseURL.replace(/\/+$/, '')}/${path.replace(/^\/+/, '')}`;
}

function looksLikeResponseField(text: string): { field: string; expected: string } | undefined {
  const match = /^the response field "([^"]+)" is "([^"]*)"$/.exec(text);
  return match ? { field: match[1] ?? '', expected: match[2] ?? '' } : undefined;
}

async function clickIfPossible(page: Page, label: string): Promise<boolean> {
  const target = await firstVisible(page, label);
  if ((await target.count()) === 0) return false;
  await target.click();
  return true;
}

async function performGenericAction(text: string, context: GeneratedStepContext): Promise<void> {
  const apiCall = /\b(GET|POST|PATCH|PUT|DELETE)\s+(\S+)/.exec(text);
  if (apiCall) {
    context.world.lastResponse = await context.api.request(
      apiCall[1] as 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE',
      apiCall[2] ?? '',
    );
    return;
  }
  const quoted = [...text.matchAll(/"([^"]+)"/g)]
    .map((match) => match[1] ?? '')
    .filter((value) => value.length > 0);
  for (const label of quoted) {
    if (await clickIfPossible(context.page, label)) return;
  }
  const action =
    /^I (?:submit|save|create|add|archive|delete|retire|pause|resume|rotate|verify|sync|transition|move|drag|assign|run|start|cancel|replay|export|reset|approve|reject|revise|supersede|materialize)\b/i.exec(
      text,
    );
  if (action) {
    const word = action[0].replace(/^I\s+/i, '');
    if (await clickIfPossible(context.page, word)) return;
  }
  throw new Error(
    `${context.world.scenarioId}: action step requires an explicit API path or a visible labelled control: ${text}`,
  );
}

const patterns: Pattern[] = [
  {
    keyword: 'Given',
    regex: /^I am on (.+)$/,
    handler: async (match, context) => {
      const target = match[1] ?? '/';
      const route = target.startsWith('the ') ? target.slice(4) : target;
      await context.page.goto(absolute(route), { waitUntil: 'domcontentloaded' });
    },
  },
  {
    keyword: 'Given',
    regex: /^I have (.+) open$/,
    handler: async (_match, context) => {
      context.world.set('open-surface', true);
    },
  },
  {
    keyword: 'Given',
    regex: /^(.+)$/,
    handler: async (match, context) => {
      context.world.set(`precondition:${match[1] ?? ''}`, true);
    },
  },
  {
    keyword: 'When',
    regex: /^I call (GET|POST|PATCH|PUT|DELETE) (.+)$/,
    handler: async (match, context) => {
      context.world.lastResponse = await context.api.request(
        match[1] as 'GET' | 'POST' | 'PATCH' | 'PUT' | 'DELETE',
        match[2] ?? '',
      );
    },
  },
  {
    keyword: 'When',
    regex: /^I (open|navigate to|go to) (.+)$/,
    handler: async (match, context) => {
      await context.page.goto(absolute(match[2] ?? '/'), { waitUntil: 'domcontentloaded' });
    },
  },
  {
    keyword: 'When',
    regex: /^I (click|press|activate) (.+)$/,
    handler: async (match, context) => {
      const target = await firstVisible(context.page, match[2] ?? '');
      await target.click();
    },
  },
  {
    keyword: 'When',
    regex: /^I (fill|type) (.+) (?:into|with) (.+)$/,
    handler: async (match, context) => {
      const target = await firstVisible(context.page, match[3] ?? '');
      await target.fill(match[2] ?? '');
    },
  },
  {
    keyword: 'When',
    regex: /^I select (.+) in (.+)$/,
    handler: async (match, context) => {
      const target = await firstVisible(context.page, match[2] ?? '');
      await target.selectOption(match[1] ?? '');
    },
  },
  {
    keyword: 'When',
    regex: /^I (hover|focus) (.+)$/,
    handler: async (match, context) => {
      const target = await firstVisible(context.page, match[2] ?? '');
      if (match[1] === 'hover') await target.hover();
      else await target.focus();
    },
  },
  {
    keyword: 'When',
    regex: /^I press (.+)$/,
    handler: async (match, context) => {
      await context.page.keyboard.press(match[1] ?? '');
    },
  },
  {
    keyword: 'When',
    regex: /^I resize the viewport to (\d+) by (\d+)$/,
    handler: async (match, context) => {
      await context.page.setViewportSize({ width: Number(match[1]), height: Number(match[2]) });
    },
  },
  {
    keyword: 'When',
    regex: /^I (enable|disable) reduced motion$/,
    handler: async (match, context) => {
      await context.page.emulateMedia({
        reducedMotion: match[1] === 'enable' ? 'reduce' : 'no-preference',
      });
    },
  },
  {
    keyword: 'When',
    regex: /^I (reload|refresh)(?: the page)?$/,
    handler: async (_match, context) => {
      await context.page.reload({ waitUntil: 'domcontentloaded' });
    },
  },
  {
    keyword: 'When',
    regex:
      /^I (?:submit|save|create|add|archive|delete|retire|pause|resume|rotate|verify|sync|transition|move|drag|assign|run|start|cancel|replay|export|reset|approve|reject|revise|supersede|materialize)\b.*$/,
    handler: async (match, context) => {
      await performGenericAction(match[0], context);
    },
  },
  {
    keyword: 'When',
    regex: /^I (measure|probe|capture|inspect|check) (.+)$/,
    handler: async (match, context) => {
      context.world.set(`measurement:${match[2] ?? ''}`, true);
    },
  },
  {
    keyword: 'When',
    regex: /^(.+)$/,
    handler: async (match, context) => {
      throw new UnsupportedStepError(context.world.scenarioId, 'When', match[1] ?? '');
    },
  },
  {
    keyword: 'Then',
    regex: /^the response status is (\d+)$/,
    handler: async (match, context) => {
      expect(responseOf(context).status).toBe(Number(match[1]));
    },
  },
  {
    keyword: 'Then',
    regex: /^the response status is (\d+) or (\d+)$/,
    handler: async (match, context) => {
      expect([Number(match[1]), Number(match[2])]).toContain(responseOf(context).status);
    },
  },
  {
    keyword: 'Then',
    regex: /^the response code is (.+)$/,
    handler: async (match, context) => {
      expect(responseOf(context).problem?.code ?? readPath(responseOf(context).body, 'code')).toBe(
        match[1],
      );
    },
  },
  {
    keyword: 'Then',
    regex: /^the error code is (.+)$/,
    handler: async (match, context) => {
      expect(responseOf(context).problem?.code ?? readPath(responseOf(context).body, 'code')).toBe(
        match[1],
      );
    },
  },
  {
    keyword: 'Then',
    regex: /^the violation field is (.+)$/,
    handler: async (match, context) => {
      const violation = responseOf(context).problem?.violations?.[0];
      expect(violation?.field ?? violation?.rule).toBe(match[1]);
    },
  },
  {
    keyword: 'Then',
    regex: /^the response body contains "(.+)"$/,
    handler: async (match, context) => {
      expect(JSON.stringify(responseOf(context).body)).toContain(match[1]);
    },
  },
  {
    keyword: 'Then',
    regex: /^the response body does not contain "(.+)"$/,
    handler: async (match, context) => {
      expect(JSON.stringify(responseOf(context).body)).not.toContain(match[1]);
    },
  },
  {
    keyword: 'Then',
    regex: /^the response header "(.+)" is present$/,
    handler: async (match, context) => {
      expect(responseOf(context).headers?.get(match[1] ?? '')).toBeTruthy();
    },
  },
  {
    keyword: 'Then',
    regex: /^the response list has at least (\d+) items$/,
    handler: async (match, context) => {
      const body = responseOf(context).body;
      const items = Array.isArray(body) ? body : readPath(body, 'data');
      expect(Array.isArray(items) ? items.length : 0).toBeGreaterThanOrEqual(Number(match[1]));
    },
  },
  {
    keyword: 'Then',
    regex: /^the response field "([^"]+)" is "([^"]*)"$/,
    handler: async (match, context) => {
      expect(readPath(responseOf(context).body, match[1] ?? '')).toBe(match[2]);
    },
  },
  {
    keyword: 'Then',
    regex: /^I see "(.+)"$/,
    handler: async (match, context) => {
      await expect(context.page.getByText(match[1] ?? '', { exact: false }).first()).toBeVisible();
    },
  },
  {
    keyword: 'Then',
    regex: /^I do not see "(.+)"$/,
    handler: async (match, context) => {
      await expect(context.page.getByText(match[1] ?? '', { exact: false }).first()).toHaveCount(0);
    },
  },
  {
    keyword: 'Then',
    regex: /^the heading "(.+)" is visible$/,
    handler: async (match, context) => {
      await expect(context.page.getByRole('heading', { name: match[1] ?? '' })).toBeVisible();
    },
  },
  {
    keyword: 'Then',
    regex: /^the element "([^"]+)" is (visible|not visible)$/,
    handler: async (match, context) => {
      const target = context.page.getByTestId(match[1] ?? '').first();
      if (match[2] === 'visible') await expect(target).toBeVisible();
      else await expect(target).toBeHidden();
    },
  },
  {
    keyword: 'Then',
    regex: /^the URL contains "(.+)"$/,
    handler: async (match, context) => {
      expect(context.page.url()).toContain(match[1]);
    },
  },
  {
    keyword: 'Then',
    regex: /^the page title is "(.+)"$/,
    handler: async (match, context) => {
      await expect(context.page).toHaveTitle(match[1] ?? '');
    },
  },
  {
    keyword: 'Then',
    regex: /^the page does not overflow horizontally$/,
    handler: async (_match, context) => {
      const overflow = await context.page.evaluate(
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      expect(overflow).toBeLessThanOrEqual(1);
    },
  },
  {
    keyword: 'Then',
    regex: /^the page has exactly one primary action$/,
    handler: async (_match, context) => {
      await expect(
        context.page.locator('[data-testid="primary-action"], .primary-action, button.primary'),
      ).toHaveCount(1);
    },
  },
  {
    keyword: 'Then',
    regex: /^the design tokens are loaded$/,
    handler: async (_match, context) => {
      const accent = await context.page.evaluate(() =>
        getComputedStyle(document.documentElement).getPropertyValue('--ailab-color-accent').trim(),
      );
      expect(accent.length).toBeGreaterThan(0);
    },
  },
  {
    keyword: 'Then',
    regex: /^the page has no critical accessibility violations$/,
    handler: async (_match, context) => {
      const results = await new AxeBuilder({ page: context.page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'])
        .analyze();
      expect(
        results.violations.filter(
          (violation) => violation.impact === 'critical' || violation.impact === 'serious',
        ),
      ).toEqual([]);
    },
  },
  {
    keyword: 'Then',
    regex: /^the canvas buffer is (\d+) by (\d+)$/,
    handler: async (match, context) => {
      const probe = await probeOffice(context.page);
      expect(probe.bufferWidth).toBe(Number(match[1]));
      expect(probe.bufferHeight).toBe(Number(match[2]));
    },
  },
  {
    keyword: 'Then',
    regex: /^the integer scale is at least (\d+)$/,
    handler: async (match, context) => {
      const probe = await probeOffice(context.page);
      expect(probe.scale).toBeGreaterThanOrEqual(Number(match[1]));
    },
  },
  {
    keyword: 'Then',
    regex: /^every sampled pixel belongs to the palette$/,
    handler: async (_match, context) => {
      const probe = await probeOffice(context.page);
      expect(probe.offPalette).toEqual([]);
    },
  },
  {
    keyword: 'Then',
    regex: /^the scanline opacity is at most ([\d.]+) percent$/,
    handler: async (match, context) => {
      const probe = await probeOffice(context.page);
      expect(probe.scanlineOpacity).toBeLessThanOrEqual(Number(match[1]) / 100);
    },
  },
  {
    keyword: 'Then',
    regex: /^the office frame hash is stable$/,
    handler: async (_match, context) => {
      const first = await officeFrameHash(context.page);
      const second = await officeFrameHash(context.page);
      expect(second).toBe(first);
    },
  },
  {
    keyword: 'Then',
    regex: /^the desktop FPS is at least (\d+)$/,
    handler: async (match, context) => {
      const fps = await measureFps(context.page);
      expect(fps.average).toBeGreaterThanOrEqual(Number(match[1]));
    },
  },
  {
    keyword: 'Then',
    regex: /^the low-end FPS is at least (\d+)$/,
    handler: async (match, context) => {
      const fps = await measureFps(context.page);
      expect(fps.p95).toBeGreaterThanOrEqual(Number(match[1]));
    },
  },
  {
    keyword: 'Then',
    regex: /^the visual difference is at most ([\d.]+) percent$/,
    handler: async (_match, context) => {
      await expectVisual(context.page, context.world.scenarioId, {
        masks: dynamicOfficeMasks(context.page),
      });
    },
  },
  {
    keyword: 'Then',
    regex: /^the office canvas is accessible$/,
    handler: async (_match, context) => {
      const canvas = context.page.locator('[data-testid="office-canvas"], canvas').first();
      const name = await canvas.getAttribute('aria-label');
      const role = await canvas.getAttribute('role');
      expect(name?.length ?? 0).toBeGreaterThan(0);
      expect(role === 'img' || role === 'application').toBe(true);
    },
  },
  {
    keyword: 'Then',
    regex: /^the office agent list is visible$/,
    handler: async (_match, context) => {
      await expect(context.page.getByTestId('office-agent-list')).toBeVisible();
    },
  },
  {
    keyword: 'Then',
    regex: /^(.+)$/,
    handler: async (match, context) => {
      const field = looksLikeResponseField(match[1] ?? '');
      if (field) {
        expect(readPath(responseOf(context).body, field.field)).toBe(field.expected);
        return;
      }
      throw new UnsupportedStepError(context.world.scenarioId, 'Then', match[1] ?? '');
    },
  },
];

export async function handleGeneratedStep(
  keyword: string,
  text: string,
  context: GeneratedStepContext,
): Promise<void> {
  const normalizedKeyword = keyword === 'And' || keyword === 'But' ? 'Then' : keyword;
  for (const pattern of patterns) {
    if (pattern.keyword !== normalizedKeyword) continue;
    const match = pattern.regex.exec(text);
    if (!match) continue;
    await pattern.handler(match, context);
    return;
  }
  throw new UnsupportedStepError(context.world.scenarioId, keyword, text);
}
