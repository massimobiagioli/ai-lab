import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { vi } from 'vitest';
import { DEMO_AGENTS, DEMO_PROJECTS, DEMO_PROVIDERS } from '../../core/demo/demo-data';
import { Inventory } from './inventory';
import { InventoryStore } from './store/inventory.store';

function fakeStore() {
  return {
    state: signal('ready'),
    agents: signal(DEMO_AGENTS),
    projects: signal(DEMO_PROJECTS),
    providers: signal(DEMO_PROVIDERS),
    error: signal(null),
    demo: signal(true),
    load: vi.fn(),
    addAgent: vi.fn(),
    addProject: vi.fn(),
    addProvider: vi.fn(),
  };
}

async function setup(section: 'agents' | 'projects' | 'providers', store = fakeStore()) {
  await TestBed.configureTestingModule({
    imports: [Inventory],
    providers: [provideRouter([]), { provide: InventoryStore, useValue: store }],
  }).compileComponents();
  const fixture = TestBed.createComponent(Inventory);
  fixture.componentRef.setInput('section', section);
  fixture.detectChanges();
  return { fixture, store };
}

describe('Inventory', () => {
  it('renders the agents and opens the detail drawer', async () => {
    const { fixture } = await setup('agents');
    const element = fixture.nativeElement as HTMLElement;
    expect(element.querySelectorAll('tbody tr').length).toBe(DEMO_AGENTS.length);
    (element.querySelector('tbody tr') as HTMLElement).click();
    fixture.detectChanges();
    expect(element.textContent).toContain('Tech Lead');
  });

  it('creates a new agent with inline validation', async () => {
    const { fixture, store } = await setup('agents');
    const element = fixture.nativeElement as HTMLElement;
    const newButton = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'New',
    );
    newButton?.click();
    fixture.detectChanges();
    const input = element.querySelector<HTMLInputElement>('input');
    if (input) {
      input.value = 'New Dev';
      input.dispatchEvent(new Event('input'));
    }
    fixture.detectChanges();
    const confirm = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Confirm',
    );
    confirm?.click();
    expect(store.addAgent).toHaveBeenCalled();
  });
});
