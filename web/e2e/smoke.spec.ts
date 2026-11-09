import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';

test('dashboard renders the shell and onboarding', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded' });
  await expect(page.getByRole('heading', { level: 1, name: 'Dashboard' })).toBeVisible();
  await expect(page.getByText('Getting started')).toBeVisible();
  await expect(page.getByRole('link', { name: 'Runs', exact: true })).toBeVisible();
});

test('primary routes render', async ({ page }) => {
  for (const [route, heading] of [
    ['/runs', 'Runs'],
    ['/projects/demo/board', 'Kanban board'],
    ['/inventory/agents', 'Agents'],
    ['/projects/demo/sow', 'SOW workspace'],
    ['/memory', 'Memory'],
    ['/settings', 'Settings'],
  ] as const) {
    await page.goto(route, { waitUntil: 'domcontentloaded' });
    await expect(page.getByRole('heading', { level: 1, name: heading })).toBeVisible();
  }
});

for (const route of ['/', '/settings', '/memory', '/design']) {
  test(`no critical a11y issues on ${route}`, async ({ page }) => {
    await page.goto(route, { waitUntil: 'domcontentloaded' });
    const results = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();
    const serious = results.violations.filter(
      (violation) => violation.impact === 'critical' || violation.impact === 'serious',
    );
    expect(serious).toEqual([]);
  });
}
