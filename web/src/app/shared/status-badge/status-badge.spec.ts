import { TestBed } from '@angular/core/testing';
import { StatusBadge } from './status-badge';

describe('StatusBadge', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [StatusBadge],
    }).compileComponents();
  });

  it('renders the label with a status role', () => {
    const fixture = TestBed.createComponent(StatusBadge);
    fixture.componentRef.setInput('label', 'READY');
    fixture.detectChanges();
    const badge = (fixture.nativeElement as HTMLElement).querySelector('.status-badge');
    expect(badge?.textContent).toContain('READY');
    expect(badge?.getAttribute('role')).toBe('status');
  });

  it('applies the tone class', () => {
    const fixture = TestBed.createComponent(StatusBadge);
    fixture.componentRef.setInput('label', 'DONE');
    fixture.componentRef.setInput('tone', 'success');
    fixture.detectChanges();
    expect(
      (fixture.nativeElement as HTMLElement).querySelector('.status-badge--success'),
    ).toBeTruthy();
  });
});
