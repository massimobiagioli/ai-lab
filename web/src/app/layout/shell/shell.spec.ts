import { TestBed } from '@angular/core/testing';
import { provideRouter } from '@angular/router';
import { Shell } from './shell';

describe('Shell', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [Shell],
      providers: [provideRouter([])],
    }).compileComponents();
  });

  it('renders the skip link, sidebar and main landmark', () => {
    const fixture = TestBed.createComponent(Shell);
    fixture.detectChanges();
    const element = fixture.nativeElement as HTMLElement;
    expect(element.querySelector('a.skip-link')?.getAttribute('href')).toBe('#main');
    expect(element.querySelector('app-sidebar')).toBeTruthy();
    expect(element.querySelector('main#main')).toBeTruthy();
  });
});
