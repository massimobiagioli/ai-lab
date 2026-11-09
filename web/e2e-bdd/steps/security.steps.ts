import { createBdd } from 'playwright-bdd';
import { test, expect } from '../support/fixtures';

const { Then } = createBdd(test);

Then('the response has the security header {string}', async ({ world }, name: string) => {
  expect(world.lastResponse?.headers.get(name)).toBeTruthy();
});

Then('the CSP is present', async ({ world }) => {
  expect(world.lastResponse?.headers.get('content-security-policy')).toBeTruthy();
});

Then('the response does not expose a secret', async ({ world }) => {
  const body = JSON.stringify(world.lastResponse?.body ?? '');
  expect(body).not.toMatch(
    /sk-[A-Za-z0-9_-]{8,}|ghp_[A-Za-z0-9]{8,}|github_pat_[A-Za-z0-9_]{8,}|Bearer\s+[A-Za-z0-9._~+/=-]{8,}/,
  );
});

Then('the webhook signature is rejected', async ({ world }) => {
  expect([401, 403]).toContain(world.lastResponse?.status);
});

Then('the sandbox denies the command', async ({ world }) => {
  expect([403, 422]).toContain(world.lastResponse?.status);
});

Then('the origin check blocks the request', async ({ world }) => {
  expect([403, 422]).toContain(world.lastResponse?.status);
});
