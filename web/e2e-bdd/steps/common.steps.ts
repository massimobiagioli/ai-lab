import AxeBuilder from '@axe-core/playwright';
import { createBdd } from 'playwright-bdd';
import type { HttpMethod } from '../support/api-client';
import { env } from '../support/env';
import { test, expect } from '../support/fixtures';
import { firstVisible, locatorFor } from '../support/selectors';

const { Given, When, Then } = createBdd(test);

function readPath(value: unknown, path: string): unknown {
  let current: unknown = value;
  for (const part of path.split('.')) {
    if (current === null || current === undefined) return undefined;
    if (Array.isArray(current)) {
      current = current[Number(part)];
    } else if (typeof current === 'object') {
      current = (current as Record<string, unknown>)[part];
    } else {
      return undefined;
    }
  }
  return current;
}

function lastResponse(world: {
  lastResponse?: {
    body: unknown;
    problem?: { code?: string; violations?: readonly { rule?: string }[] };
  };
}): { body: unknown; problem?: { code?: string; violations?: readonly { rule?: string }[] } } {
  if (!world.lastResponse) throw new Error('no API response recorded; use "I call ..." first');
  return world.lastResponse;
}

Given('the prepared AI-Lab environment is available', async ({ api }) => {
  const response = await api.health();
  expect(response.status, 'health/live must be reachable before the BDD suite starts').toBe(200);
});

Given('the deterministic demo seed is loaded', async ({ api }) => {
  const response = await api.get('/api/v1/labs');
  expect(response.status).toBe(200);
});

When(
  /^I call (GET|POST|PATCH|PUT|DELETE) (.+)$/,
  async ({ api, world }, method: string, target: string) => {
    const path = target.replace(/^"(.*)"$/, '$1');
    world.lastResponse = await api.request(method as HttpMethod, path);
  },
);

When(
  'I call {word} {string} with body {string}',
  async ({ api, world }, method: string, path: string, body: string) => {
    world.lastResponse = await api.request(method.toUpperCase() as HttpMethod, path, {
      body: JSON.parse(body) as unknown,
    });
  },
);

When(
  'I call {word} {string} with the fixture body {string}',
  async ({ api, world }, method: string, path: string, key: string) => {
    world.lastResponse = await api.request(method.toUpperCase() as HttpMethod, path, {
      body: world.require(key),
    });
  },
);

Then('the response status is {int}', async ({ world }, status: number) => {
  expect(lastResponse(world).body, 'response body').toBeDefined();
  expect(world.lastResponse?.status).toBe(status);
});

Then('the response status is {int} or {int}', async ({ world }, first: number, second: number) => {
  expect([first, second]).toContain(world.lastResponse?.status);
});

Then('the response field {string} is {string}', async ({ world }, path: string, value: string) => {
  expect(readPath(lastResponse(world).body, path)).toBe(value);
});

Then('the response field {string} is {int}', async ({ world }, path: string, value: number) => {
  expect(readPath(lastResponse(world).body, path)).toBe(value);
});

Then('the response field {string} is {float}', async ({ world }, path: string, value: number) => {
  expect(readPath(lastResponse(world).body, path)).toBe(value);
});

Then('the response field {string} is true', async ({ world }, path: string) => {
  expect(readPath(lastResponse(world).body, path)).toBe(true);
});

Then('the response field {string} is false', async ({ world }, path: string) => {
  expect(readPath(lastResponse(world).body, path)).toBe(false);
});

Then('the response field {string} is null', async ({ world }, path: string) => {
  expect(readPath(lastResponse(world).body, path)).toBeNull();
});

Then('the response field {string} is not null', async ({ world }, path: string) => {
  expect(readPath(lastResponse(world).body, path)).not.toBeNull();
});

Then('the response code is {string}', async ({ world }, code: string) => {
  const response = lastResponse(world);
  expect(response.problem?.code ?? readPath(response.body, 'code')).toBe(code);
});

Then('the response is a Problem Details with code {string}', async ({ world }, code: string) => {
  const response = lastResponse(world);
  expect(response.problem).toBeDefined();
  expect(response.problem?.code).toBe(code);
});

Then('the response has a violation {string}', async ({ world }, rule: string) => {
  const response = lastResponse(world);
  expect(response.problem?.violations?.some((violation) => violation.rule === rule)).toBe(true);
});

Then('the response body contains {string}', async ({ world }, text: string) => {
  expect(JSON.stringify(lastResponse(world).body)).toContain(text);
});

Then('the response body does not contain {string}', async ({ world }, text: string) => {
  expect(JSON.stringify(lastResponse(world).body)).not.toContain(text);
});

Then('the response header {string} is present', async ({ world }, name: string) => {
  expect(world.lastResponse?.headers.get(name)).toBeTruthy();
});

Then(
  'the response header {string} contains {string}',
  async ({ world }, name: string, value: string) => {
    expect(world.lastResponse?.headers.get(name) ?? '').toContain(value);
  },
);

When('I open {string}', async ({ page }, target: string) => {
  const url = target.startsWith('http')
    ? target
    : `${env.baseURL.replace(/\/+$/, '')}/${target.replace(/^\/+/, '')}`;
  await page.goto(url, { waitUntil: 'domcontentloaded' });
});

When('I reload the page', async ({ page }) => {
  await page.reload({ waitUntil: 'domcontentloaded' });
});

When('I click {string}', async ({ page }, reference: string) => {
  await (await firstVisible(page, reference)).click();
});

When('I hover {string}', async ({ page }, reference: string) => {
  await (await firstVisible(page, reference)).hover();
});

When('I fill {string} with {string}', async ({ page }, reference: string, value: string) => {
  await (await firstVisible(page, reference)).fill(value);
});

When('I select {string} in {string}', async ({ page }, value: string, reference: string) => {
  await (await firstVisible(page, reference)).selectOption(value);
});

When('I press {string}', async ({ page }, key: string) => {
  await page.keyboard.press(key);
});

Then('I see {string}', async ({ page }, text: string) => {
  await expect(page.getByText(text, { exact: false }).first()).toBeVisible();
});

Then('I do not see {string}', async ({ page }, text: string) => {
  await expect(page.getByText(text, { exact: false }).first()).toHaveCount(0);
});

Then('the heading {string} is visible', async ({ page }, name: string) => {
  await expect(page.getByRole('heading', { name })).toBeVisible();
});

Then('the URL contains {string}', async ({ page }, fragment: string) => {
  expect(page.url()).toContain(fragment);
});

Then('the page title is {string}', async ({ page }, title: string) => {
  await expect(page).toHaveTitle(title);
});

Then('the element {string} is visible', async ({ page }, reference: string) => {
  await expect(locatorFor(page, reference).first()).toBeVisible();
});

Then('the element {string} is not visible', async ({ page }, reference: string) => {
  await expect(locatorFor(page, reference).first()).toBeHidden();
});

Then(
  'the element {string} has text {string}',
  async ({ page }, reference: string, text: string) => {
    await expect(locatorFor(page, reference).first()).toContainText(text);
  },
);

Then(
  'the element {string} has attribute {string} with value {string}',
  async ({ page }, reference: string, name: string, value: string) => {
    await expect(locatorFor(page, reference).first()).toHaveAttribute(name, value);
  },
);

Then('the element {string} has count {int}', async ({ page }, reference: string, count: number) => {
  await expect(locatorFor(page, reference)).toHaveCount(count);
});

Then('the page does not overflow horizontally', async ({ page }) => {
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

When('I resize the viewport to {int} by {int}', async ({ page }, width: number, height: number) => {
  await page.setViewportSize({ width, height });
});

Then('the viewport is {int} by {int}', async ({ page }, width: number, height: number) => {
  const viewport = page.viewportSize();
  expect(viewport?.width).toBe(width);
  expect(viewport?.height).toBe(height);
});

Then('the page has no critical accessibility violations', async ({ page }) => {
  const results = await new AxeBuilder({ page })
    .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'])
    .analyze();
  const serious = results.violations.filter(
    (violation) => violation.impact === 'critical' || violation.impact === 'serious',
  );
  expect(serious).toEqual([]);
});

Then('the page has exactly one primary action', async ({ page }) => {
  const primary = page.locator('[data-testid="primary-action"], .primary-action, button.primary');
  await expect(primary).toHaveCount(1);
});

Then('the design tokens are loaded', async ({ page }) => {
  const accent = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue('--ailab-color-accent').trim(),
  );
  expect(accent.length).toBeGreaterThan(0);
});
