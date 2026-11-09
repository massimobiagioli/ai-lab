import { createBdd } from 'playwright-bdd';
import { test, expect } from '../support/fixtures';

const { Then } = createBdd(test);

Then('the toast is visible', async ({ page }) => {
  await expect(page.getByTestId('toast').first()).toBeVisible();
});

Then('the drawer is open', async ({ page }) => {
  await expect(page.getByTestId('drawer').first()).toBeVisible();
});

Then('the dialog is open', async ({ page }) => {
  await expect(page.getByTestId('dialog').first()).toBeVisible();
});

Then('the focus is on {string}', async ({ page }, testId: string) => {
  const active = await page.evaluate(
    () => document.activeElement?.getAttribute('data-testid') ?? '',
  );
  expect(active).toBe(testId.replace(/^testid:/, ''));
});

Then('the page is in the {word} state', async ({ page }, state: string) => {
  await expect(page.getByTestId(`state-${state}`).first()).toBeVisible();
});

Then('the empty state is visible', async ({ page }) => {
  await expect(page.getByTestId('empty-state').first()).toBeVisible();
});

Then('the loading state is visible', async ({ page }) => {
  await expect(page.getByTestId('loading-state').first()).toBeVisible();
});

Then('the error state is visible', async ({ page }) => {
  await expect(page.getByTestId('error-state').first()).toBeVisible();
});

Then('the page has a skip link', async ({ page }) => {
  await expect(page.getByRole('link', { name: /skip/i }).first()).toBeVisible();
});
