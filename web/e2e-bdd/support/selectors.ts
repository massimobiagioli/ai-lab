import type { Locator, Page } from '@playwright/test';

export function locatorFor(page: Page, reference: string): Locator {
  const value = reference.trim();
  if (value.startsWith('testid:')) return page.getByTestId(value.slice('testid:'.length));
  if (value.startsWith('css:')) return page.locator(value.slice('css:'.length));
  if (value.startsWith('text:')) return page.getByText(value.slice('text:'.length));
  if (value.startsWith('label:')) return page.getByLabel(value.slice('label:'.length));
  if (value.startsWith('placeholder:'))
    return page.getByPlaceholder(value.slice('placeholder:'.length));
  if (value.startsWith('role:')) {
    const [, role, name] = value.split(':', 3);
    const options = name ? { name } : {};
    if (role === 'button') return page.getByRole('button', options);
    if (role === 'link') return page.getByRole('link', options);
    if (role === 'heading') return page.getByRole('heading', options);
    if (role === 'tab') return page.getByRole('tab', options);
    if (role === 'menuitem') return page.getByRole('menuitem', options);
  }
  const testId = page.getByTestId(value);
  return testId;
}

export async function firstVisible(page: Page, reference: string): Promise<Locator> {
  const testId = locatorFor(page, reference);
  if ((await testId.count()) > 0) return testId.first();
  const text = page.getByText(reference, { exact: false });
  if ((await text.count()) > 0) return text.first();
  const button = page.getByRole('button', { name: reference });
  if ((await button.count()) > 0) return button.first();
  const link = page.getByRole('link', { name: reference });
  if ((await link.count()) > 0) return link.first();
  return testId.first();
}

export const selectors = {
  officeCanvas: 'testid:office-canvas',
  officeFps: 'testid:office-fps',
  agentList: 'testid:office-agent-list',
  primaryAction: 'testid:primary-action',
  pageHeading: 'role:heading:level 1',
  toast: 'testid:toast',
  drawer: 'testid:drawer',
  dialog: 'testid:dialog',
} as const;
