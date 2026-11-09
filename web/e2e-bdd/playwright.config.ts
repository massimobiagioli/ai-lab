import { join } from 'node:path';
import { defineConfig, devices } from '@playwright/test';
import { defineBddConfig } from 'playwright-bdd';
import { env } from './support/env';

if (process.env['AILAB_BDD_VIA_MAKE'] !== '1') {
  throw new Error("bdd: run this suite only via 'make bdd'.");
}

const testDir = defineBddConfig({
  features: join(__dirname, 'features/**/*.feature'),
  steps: [join(__dirname, 'steps/**/*.ts'), join(__dirname, 'support/fixtures.ts')],
  outputDir: join(__dirname, '.features-gen'),
});

export default defineConfig({
  testDir,
  outputDir: join(__dirname, 'reports/test-results'),
  snapshotPathTemplate: join(__dirname, 'visual-baselines/{arg}{ext}'),
  fullyParallel: false,
  workers: 1,
  retries: 0,
  forbidOnly: true,
  timeout: 120_000,
  expect: { timeout: 10_000 },
  reporter: [
    ['list'],
    ['html', { outputFolder: join(__dirname, 'reports/html'), open: 'never' }],
    ['junit', { outputFile: join(__dirname, 'reports/junit.xml') }],
    [join(__dirname, 'support/json-reporter.ts')],
  ],
  use: {
    baseURL: env.baseURL,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    viewport: env.viewport,
    timezoneId: 'UTC',
    locale: 'en-US',
    reducedMotion: env.reducedMotion,
    colorScheme: 'dark',
    ignoreHTTPSErrors: false,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'], viewport: env.viewport },
    },
  ],
});
