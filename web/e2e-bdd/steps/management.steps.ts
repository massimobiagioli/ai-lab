import { createBdd } from 'playwright-bdd';
import { env } from '../support/env';
import { test, expect } from '../support/fixtures';

const { When, Then } = createBdd(test);

function absolute(path: string): string {
  if (/^https?:\/\//.test(path)) return path;
  return `${env.baseURL.replace(/\/+$/, '')}/${path.replace(/^\/+/, '')}`;
}

Then('the management route {string} has one primary action', async ({ page }, route: string) => {
  await page.goto(absolute(route), { waitUntil: 'domcontentloaded' });
  await expect(
    page.locator('[data-testid="primary-action"], .primary-action, button.primary'),
  ).toHaveCount(1);
});

Then(
  'the management route {string} shows the heading {string}',
  async ({ page }, route: string, heading: string) => {
    await page.goto(absolute(route), { waitUntil: 'domcontentloaded' });
    await expect(page.getByRole('heading', { level: 1, name: heading })).toBeVisible();
  },
);

Then('the management page has exactly one h1', async ({ page }) => {
  await expect(page.locator('h1')).toHaveCount(1);
});

Then('the management page has a main landmark', async ({ page }) => {
  await expect(page.locator('main')).toHaveCount(1);
});

Then('the element {string} uses a hard pixel shadow', async ({ page }, reference: string) => {
  const shadow = await page
    .getByTestId(reference.replace(/^testid:/, ''))
    .first()
    .evaluate((element) => getComputedStyle(element).boxShadow);
  expect(shadow === 'none' || / 0(px)?( 0(px)?)?$/.test(shadow)).toBe(true);
});

Then(
  'the element {string} follows the {int} pixel spacing scale',
  async ({ page }, reference: string, scale: number) => {
    const values = await page
      .getByTestId(reference.replace(/^testid:/, ''))
      .first()
      .evaluate((element) => {
        const style = getComputedStyle(element);
        return [
          style.marginTop,
          style.marginRight,
          style.marginBottom,
          style.marginLeft,
          style.paddingTop,
          style.paddingRight,
          style.paddingBottom,
          style.paddingLeft,
          style.gap,
        ]
          .map((value) => Number.parseFloat(value))
          .filter((value) => Number.isFinite(value));
      });
    for (const value of values) expect(value % scale).toBe(0);
  },
);

Then('the management table is semantic', async ({ page }) => {
  const table = page.locator('table').first();
  await expect(table).toBeVisible();
  await expect(table.locator('thead')).toHaveCount(1);
  await expect(table.locator('th').first()).toBeVisible();
});

When('I open the management route {string}', async ({ page }, route: string) => {
  await page.goto(absolute(route), { waitUntil: 'domcontentloaded' });
});
