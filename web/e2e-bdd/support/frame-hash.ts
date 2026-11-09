import { createHash } from 'node:crypto';
import type { Page } from '@playwright/test';

export async function officeFrameHash(page: Page): Promise<string> {
  const payload = await page.evaluate(() => {
    const hook = window.__ailabOffice;
    if (hook?.frameHash) return hook.frameHash();
    const canvas = document.querySelector<HTMLCanvasElement>(
      '[data-testid="office-canvas"], canvas.office-canvas, canvas',
    );
    if (!canvas) throw new Error('office canvas not found');
    return canvas.toDataURL('image/png');
  });
  return createHash('sha256').update(payload).digest('hex');
}
