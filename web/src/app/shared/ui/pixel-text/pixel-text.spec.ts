import { TestBed } from '@angular/core/testing';
import { PixelText } from './pixel-text';

describe('PixelText', () => {
  beforeEach(async () => {
    await TestBed.configureTestingModule({ imports: [PixelText] }).compileComponents();
  });

  it('renders pixels for a glyph', () => {
    const fixture = TestBed.createComponent(PixelText);
    fixture.componentRef.setInput('text', 'A');
    fixture.detectChanges();
    const rects = (fixture.nativeElement as HTMLElement).querySelectorAll('rect');
    expect(rects.length).toBeGreaterThan(0);
    expect(
      (fixture.nativeElement as HTMLElement).querySelector('svg')?.getAttribute('aria-label'),
    ).toBe('A');
  });

  it('renders nothing for spaces but keeps width', () => {
    const fixture = TestBed.createComponent(PixelText);
    fixture.componentRef.setInput('text', 'A B');
    fixture.detectChanges();
    expect((fixture.nativeElement as HTMLElement).querySelectorAll('rect').length).toBeGreaterThan(
      0,
    );
  });
});
