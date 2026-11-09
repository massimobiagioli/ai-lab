import { expect } from '@playwright/test';
import { test as base } from 'playwright-bdd';
import { ApiClient } from './api-client';
import { env } from './env';
import { World } from './world';

export interface BddFixtures {
  readonly api: ApiClient;
  readonly world: World;
}

export const test = base.extend<BddFixtures>({
  // eslint-disable-next-line no-empty-pattern
  api: async ({}, use) => {
    await use(new ApiClient(env.baseURL, env.apiToken));
  },
  // eslint-disable-next-line no-empty-pattern
  world: async ({}, use, testInfo) => {
    await use(new World(testInfo));
  },
});

export { expect };
