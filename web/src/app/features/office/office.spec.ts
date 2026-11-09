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
    const agents = element.querySelectorAll('[data-testid^="agent-"]');
    expect(agents.length).toBe(5);
    expect(element.textContent).toContain('No agent selected');
  });

  it('selects an agent', () => {
    const fixture = TestBed.createComponent(Office);
    fixture.detectChanges();
    const button = (fixture.nativeElement as HTMLElement).querySelector<HTMLButtonElement>(
      '[data-testid^="agent-"]',
    );
    button?.click();
    fixture.detectChanges();
    expect((fixture.nativeElement as HTMLElement).textContent).toContain('Animation:');
  });

  it('opens an object panel from the object list', () => {
    const fixture = TestBed.createComponent(Office);
    fixture.detectChanges();
    const objects = (fixture.nativeElement as HTMLElement).querySelectorAll(
      '[data-testid^="object-"]',
    );
    expect(objects.length).toBe(7);
    (objects[0] as HTMLButtonElement).click();
    fixture.detectChanges();
    expect((fixture.nativeElement as HTMLElement).textContent).toContain('Kanban board');
  });
});
