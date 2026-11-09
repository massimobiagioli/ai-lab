import { signal } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { ActivatedRoute, convertToParamMap, provideRouter } from '@angular/router';
import { WsService } from '../../core/ws/ws.service';
import { Office } from './office';

describe('Office deep links', () => {
  it('restores agent focus and the CV panel', async () => {
    const route = {
      snapshot: { queryParamMap: convertToParamMap({ agent: 'agent-dev', panel: 'cv' }) },
    };
    await TestBed.configureTestingModule({
      imports: [Office],
      providers: [
        provideRouter([]),
        { provide: ActivatedRoute, useValue: route },
        { provide: WsService, useValue: { lastEvent: signal(null) } },
      ],
    }).compileComponents();
    const fixture = TestBed.createComponent(Office);
    fixture.detectChanges();
    await Promise.resolve();
    const button = (fixture.nativeElement as HTMLElement).querySelector<HTMLButtonElement>(
      '[data-testid="agent-agent-dev"]',
    );
    expect(document.activeElement).toBe(button);
    expect((fixture.nativeElement as HTMLElement).textContent).toContain('CV:');
  });
});
