import { TestBed } from '@angular/core/testing';
import { Dashboard, FlowMetrics } from './dashboard';

const metrics: FlowMetrics = {
  leadTimeP50Minutes: 30,
  leadTimeP95Minutes: 120,
  cycleTimeP50Minutes: 20,
  wip: 3,
  throughputWeekly: 24,
  blockedShare: 0.05,
};

describe('Dashboard', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [Dashboard] }).compileComponents();
  });

  it('renders the flow metrics', () => {
    const fixture = TestBed.createComponent(Dashboard);
    fixture.componentRef.setInput('metrics', metrics);
    fixture.detectChanges();
    const text = (fixture.nativeElement as HTMLElement).textContent ?? '';
    expect(text).toContain('Lead p50');
    expect(text).toContain('30 min');
    expect(text).toContain('24 / week');
  });

  it('marks unhealthy flow', () => {
    const fixture = TestBed.createComponent(Dashboard);
    fixture.componentRef.setInput('metrics', { ...metrics, blockedShare: 0.25 });
    fixture.detectChanges();
    expect(
      (fixture.nativeElement as HTMLElement).querySelector('.status-badge--warning'),
    ).toBeTruthy();
  });
});
