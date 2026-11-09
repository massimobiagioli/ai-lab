import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { WsService } from '../../core/ws/ws.service';
import type { EventEnvelope } from '../../domain/models';
import { Office } from './office';

describe('Office WS bridge', () => {
  it('updates an agent state from lab.agent.status_changed', async () => {
    const lastEvent = signal<EventEnvelope | null>(null);
    await TestBed.configureTestingModule({
      imports: [Office],
      providers: [provideRouter([]), { provide: WsService, useValue: { lastEvent } }],
    }).compileComponents();
    const fixture = TestBed.createComponent(Office);
    fixture.detectChanges();
    const button = (fixture.nativeElement as HTMLElement).querySelector<HTMLButtonElement>(
      '[data-testid="agent-agent-dev"]',
    );
    button?.click();
    fixture.detectChanges();
    lastEvent.set({
      event_id: 'evt-1',
      event: 'lab.agent.status_changed',
      seq: 1,
      lab_id: 'lab-1',
      occurred_at: '2026-11-09T00:00:00Z',
      payload: { agent_id: 'agent-dev', from: 'idle', to: 'working' },
    });
    await fixture.whenStable();
    fixture.detectChanges();
    await new Promise((resolve) => setTimeout(resolve, 0));
    fixture.detectChanges();
    expect((fixture.nativeElement as HTMLElement).textContent).toContain('working');
  });
});
