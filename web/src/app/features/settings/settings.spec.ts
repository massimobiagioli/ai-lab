import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { vi } from 'vitest';
import { DEMO_SETTINGS } from '../../core/demo/demo-data';
import { Settings } from './settings';
import { SettingsStore } from './store/settings.store';

function fakeStore() {
  return {
    state: signal('ready'),
    settings: signal(DEMO_SETTINGS),
    error: signal(null),
    demo: signal(true),
    load: vi.fn(),
  };
}

async function setup() {
  await TestBed.configureTestingModule({
    imports: [Settings],
    providers: [provideRouter([]), { provide: SettingsStore, useValue: fakeStore() }],
  }).compileComponents();
  const fixture = TestBed.createComponent(Settings);
  fixture.detectChanges();
  return fixture;
}

describe('Settings', () => {
  it('validates the budget and saves', async () => {
    const fixture = await setup();
    const element = fixture.nativeElement as HTMLElement;
    const inputs = element.querySelectorAll<HTMLInputElement>('input');
    const budget = inputs[0];
    if (budget) {
      budget.value = '-1';
      budget.dispatchEvent(new Event('input'));
    }
    fixture.detectChanges();
    const save = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Save',
    );
    save?.click();
    fixture.detectChanges();
    expect(element.textContent).toContain('Budget must be a non-negative number');
  });

  it('archives the lab after confirmation', async () => {
    const fixture = await setup();
    const element = fixture.nativeElement as HTMLElement;
    const archive = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Archive lab',
    );
    archive?.click();
    fixture.detectChanges();
    const confirm = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Confirm',
    );
    confirm?.click();
    fixture.detectChanges();
    expect(element.textContent).toContain('Lab archived in demo mode');
  });
});
