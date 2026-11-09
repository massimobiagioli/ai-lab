import type { Locator, Page } from '@playwright/test';
import { expect } from '@playwright/test';
import { env } from './env';

export interface VisualOptions {
  readonly masks?: readonly Locator[];
  readonly fullPage?: boolean;
  readonly animations?: 'disabled' | 'allow';
}

export async function expectVisual(
  page: Page,
  name: string,
  options: VisualOptions = {},
): Promise<void> {
  await expect(page).toHaveScreenshot(`${name}.png`, {
    maxDiffPixelRatio: env.visualThreshold,
    animations: options.animations ?? 'disabled',
    fullPage: options.fullPage ?? false,
    ...(options.masks ? { mask: [...options.masks] } : {}),
  });
}

export async function capture(page: Page, name: string, fullPage = false): Promise<string> {
  const path = `${env.screenshotsDir}/${name}.png`;
  await page.screenshot({ path, fullPage, animations: 'disabled' });
  return path;
}

export function dynamicOfficeMasks(page: Page): Locator[] {
  return [
    page.getByTestId('office-fps'),
    page.getByTestId('office-clock'),
    page.getByTestId('office-toast'),
  ];
}
