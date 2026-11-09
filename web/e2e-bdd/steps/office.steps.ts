import { createBdd } from 'playwright-bdd';
import { test, expect } from '../support/fixtures';
import { officeFrameHash } from '../support/frame-hash';
import { measureFps, probeOffice } from '../support/office-probe';
import { dynamicOfficeMasks, expectVisual } from '../support/visual';

const { When, Then } = createBdd(test);

Then('the canvas buffer is {int} by {int}', async ({ page }, width: number, height: number) => {
  const probe = await probeOffice(page);
  expect(probe.found, 'office canvas must exist').toBe(true);
  expect(probe.bufferWidth).toBe(width);
  expect(probe.bufferHeight).toBe(height);
});

Then('the integer scale is at least {int}', async ({ page }, minimum: number) => {
  const probe = await probeOffice(page);
  expect(probe.scale).toBeGreaterThanOrEqual(minimum);
  expect(Math.abs(probe.scale - Math.round(probe.scale))).toBeLessThan(0.0001);
});

Then('every sampled pixel belongs to the palette', async ({ page }) => {
  const probe = await probeOffice(page);
  expect(probe.error, `canvas sampling failed: ${probe.error ?? ''}`).toBeUndefined();
  expect(probe.offPalette, `off-palette colours: ${probe.offPalette.join(', ')}`).toEqual([]);
});

Then('the scanline opacity is at most {float} percent', async ({ page }, percent: number) => {
  const probe = await probeOffice(page);
  expect(probe.scanlineOpacity).toBeLessThanOrEqual(percent / 100);
});

Then('the office frame hash is stable', async ({ page }) => {
  const first = await officeFrameHash(page);
  const second = await officeFrameHash(page);
  expect(second).toBe(first);
});

Then('the office frame hash is recorded', async ({ page, world }) => {
  world.set('frameHash', await officeFrameHash(page));
});

Then('the desktop FPS is at least {int}', async ({ page }, minimum: number) => {
  const fps = await measureFps(page, 2000);
  expect(fps.average).toBeGreaterThanOrEqual(minimum);
});

Then('the low-end FPS is at least {int}', async ({ page }, minimum: number) => {
  const fps = await measureFps(page, 2000);
  expect(fps.p95).toBeGreaterThanOrEqual(minimum);
});

Then('the visual difference is at most {float} percent', async ({ page, world }) => {
  await expectVisual(page, world.scenarioId, { masks: dynamicOfficeMasks(page) });
});

When('I enable reduced motion', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
});

When('I disable reduced motion', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'no-preference' });
});

Then('the office canvas is accessible', async ({ page }) => {
  const canvas = page.locator('[data-testid="office-canvas"], canvas').first();
  const name = await canvas.getAttribute('aria-label');
  const role = await canvas.getAttribute('role');
  expect(name?.length ?? 0).toBeGreaterThan(0);
  expect(role === 'img' || role === 'application').toBe(true);
});

Then('the office agent list is visible', async ({ page }) => {
  await expect(page.getByTestId('office-agent-list')).toBeVisible();
});
