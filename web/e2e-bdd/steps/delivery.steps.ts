import { createBdd } from 'playwright-bdd';
import { test, expect } from '../support/fixtures';

const { Then } = createBdd(test);

function field(world: { lastResponse?: { body: unknown } }, path: string): unknown {
  let current: unknown = world.lastResponse?.body;
  for (const part of path.split('.')) {
    if (current === null || current === undefined) return undefined;
    if (Array.isArray(current)) current = current[Number(part)];
    else if (typeof current === 'object') current = (current as Record<string, unknown>)[part];
    else return undefined;
  }
  return current;
}

Then('the SOW status is {word}', async ({ world }, status: string) => {
  expect(field(world, 'status')).toBe(status);
});

Then('the SOW has {int} sections', async ({ world }, count: number) => {
  const sections = field(world, 'sections');
  expect(Array.isArray(sections) ? sections.length : 0).toBe(count);
});

Then('the plan is acyclic', async ({ world }) => {
  expect(field(world, 'acyclic')).toBe(true);
});

Then('the plan has {int} tasks', async ({ world }, count: number) => {
  const tasks = field(world, 'tasks');
  expect(Array.isArray(tasks) ? tasks.length : 0).toBe(count);
});

Then('the run status is {word}', async ({ world }, status: string) => {
  expect(field(world, 'status')).toBe(status);
});

Then('the run has {int} steps', async ({ world }, count: number) => {
  const steps = field(world, 'steps');
  expect(Array.isArray(steps) ? steps.length : 0).toBe(count);
});

Then('the run cost is at most {float}', async ({ world }, cost: number) => {
  expect(Number(field(world, 'cost_usd') ?? 0)).toBeLessThanOrEqual(cost);
});

Then('the agent status is {word}', async ({ world }, status: string) => {
  expect(field(world, 'status')).toBe(status);
});

Then('the agent role is {word}', async ({ world }, role: string) => {
  expect(field(world, 'role')).toBe(role);
});

Then('the agent CV version is {int}', async ({ world }, version: number) => {
  expect(field(world, 'cv_version') ?? field(world, 'version')).toBe(version);
});
