import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { Office } from './office';

describe('Office', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Office],
      providers: [provideRouter([])],
    }).compileComponents();
  });

  it('lists the agents and starts unselected', () => {
    const fixture = TestBed.createComponent(Office);
    fixture.detectChanges();
    const element = fixture.nativeElement as HTMLElement;
    const agents = element.querySelectorAll('.office__agent');
    expect(agents.length).toBe(5);
    expect(element.textContent).toContain('No agent selected');
  });

  it('selects an agent', () => {
    const fixture = TestBed.createComponent(Office);
    fixture.detectChanges();
    const button = (fixture.nativeElement as HTMLElement).querySelector<HTMLButtonElement>(
      '.office__agent',
    );
    button?.click();
    fixture.detectChanges();
    expect((fixture.nativeElement as HTMLElement).textContent).toContain('Animation:');
  });
});
