import { createBdd } from 'playwright-bdd';
import { test, expect } from '../support/fixtures';

const { Then } = createBdd(test);

function items(body: unknown): unknown[] {
  if (Array.isArray(body)) return body;
  if (body && typeof body === 'object') {
    const data = (body as Record<string, unknown>)['data'];
    if (Array.isArray(data)) return data;
    const hits = (body as Record<string, unknown>)['hits'];
    if (Array.isArray(hits)) return hits;
  }
  return [];
}

Then('the response list has at least {int} items', async ({ world }, minimum: number) => {
  expect(items(world.lastResponse?.body).length).toBeGreaterThanOrEqual(minimum);
});

Then('the response list is not empty', async ({ world }) => {
  expect(items(world.lastResponse?.body).length).toBeGreaterThan(0);
});

Then('the response list is empty', async ({ world }) => {
  expect(items(world.lastResponse?.body).length).toBe(0);
});

Then('the response list has exactly {int} items', async ({ world }, count: number) => {
  expect(items(world.lastResponse?.body).length).toBe(count);
});

Then('the response has an id', async ({ world }) => {
  const body = world.lastResponse?.body;
  expect(
    body && typeof body === 'object' ? (body as Record<string, unknown>)['id'] : undefined,
  ).toBeTruthy();
});

Then('the response id is a UUID', async ({ world }) => {
  const body = world.lastResponse?.body as Record<string, unknown> | undefined;
  expect(String(body?.['id'] ?? '')).toMatch(
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
  );
});
