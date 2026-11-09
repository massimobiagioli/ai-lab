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

Then('the card is in {word}', async ({ world }, column: string) => {
  expect(field(world, 'column') ?? field(world, 'card.column')).toBe(column);
});

Then('the card column is {word}', async ({ world }, column: string) => {
  expect(field(world, 'column') ?? field(world, 'card.column')).toBe(column);
});

Then('the card key matches {string}', async ({ world }, key: string) => {
  expect(field(world, 'key') ?? field(world, 'card.key')).toBe(key);
});

Then('the card has a parent link', async ({ world }) => {
  expect(field(world, 'parent_id') ?? field(world, 'card.parent_id')).toBeTruthy();
});

Then('the card has version {int}', async ({ world }, version: number) => {
  expect(field(world, 'version') ?? field(world, 'card.version')).toBe(version);
});

Then('the board has exactly {int} columns', async ({ world }, count: number) => {
  const columns = field(world, 'columns');
  expect(Array.isArray(columns) ? columns.length : 0).toBe(count);
});

Then('the transition is denied with {string}', async ({ world }, code: string) => {
  const problem = world.lastResponse?.problem;
  expect(problem?.code).toBe(code);
});

Then('the transition is allowed', async ({ world }) => {
  expect(world.lastResponse?.status).toBe(200);
});

Then('the card is assigned to {string}', async ({ world }, agentId: string) => {
  expect(field(world, 'assignee_id')).toBe(agentId);
});

Then('the card has {int} acceptance criteria', async ({ world }, count: number) => {
  const criteria = field(world, 'story.acceptance_criteria') ?? field(world, 'acceptance_criteria');
  expect(Array.isArray(criteria) ? criteria.length : 0).toBe(count);
});
