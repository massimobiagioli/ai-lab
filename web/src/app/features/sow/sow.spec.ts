import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { vi } from 'vitest';
import { DEMO_SOW } from '../../core/demo/demo-data';
import { Sow } from './sow';
import { SowStore } from './store/sow.store';

function fakeStore() {
  return {
    state: signal('ready'),
    sow: signal(DEMO_SOW),
    error: signal(null),
    demo: signal(true),
    load: vi.fn(),
  };
}

describe('Sow', () => {
  it('renders the sections and delivery graph', async () => {
    await TestBed.configureTestingModule({
      imports: [Sow],
      providers: [provideRouter([]), { provide: SowStore, useValue: fakeStore() }],
    }).compileComponents();
    const fixture = TestBed.createComponent(Sow);
    fixture.componentRef.setInput('projectId', 'demo');
    fixture.detectChanges();
    const element = fixture.nativeElement as HTMLElement;
    expect(element.textContent).toContain('Goal');
    expect(element.textContent).toContain('Data layer');
    expect(element.querySelectorAll('.task').length).toBe(DEMO_SOW.tasks.length);
  });

  it('confirms materialization', async () => {
    await TestBed.configureTestingModule({
      imports: [Sow],
      providers: [provideRouter([]), { provide: SowStore, useValue: fakeStore() }],
    }).compileComponents();
    const fixture = TestBed.createComponent(Sow);
    fixture.componentRef.setInput('projectId', 'demo');
    fixture.detectChanges();
    const element = fixture.nativeElement as HTMLElement;
    const materialize = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Materialize',
    );
    materialize?.click();
    fixture.detectChanges();
    const confirm = [...element.querySelectorAll('button')].find(
      (button) => button.textContent?.trim() === 'Confirm',
    );
    confirm?.click();
    fixture.detectChanges();
    expect(element.textContent).toContain('Materialized in demo mode');
  });
});
