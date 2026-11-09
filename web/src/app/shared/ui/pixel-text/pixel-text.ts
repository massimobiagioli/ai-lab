import { Component, computed, input } from '@angular/core';

/** 3x5 pixel glyphs; `#` is an opaque pixel. */
const GLYPHS: Record<string, readonly string[]> = {
  A: ['###', '#.#', '###', '#.#', '#.#'],
  B: ['###', '#.#', '##.', '#.#', '###'],
  C: ['###', '#..', '#..', '#..', '###'],
  D: ['##.', '#.#', '#.#', '#.#', '##.'],
  E: ['###', '#..', '##.', '#..', '###'],
  F: ['###', '#..', '##.', '#..', '#..'],
  G: ['###', '#..', '#.#', '#.#', '###'],
  H: ['#.#', '#.#', '###', '#.#', '#.#'],
  I: ['###', '.#.', '.#.', '.#.', '###'],
  J: ['..#', '..#', '..#', '#.#', '###'],
  K: ['#.#', '#.#', '##.', '#.#', '#.#'],
  L: ['#..', '#..', '#..', '#..', '###'],
  M: ['#.#', '###', '###', '#.#', '#.#'],
  N: ['##.', '#.#', '#.#', '#.#', '#.#'],
  O: ['###', '#.#', '#.#', '#.#', '###'],
  P: ['###', '#.#', '###', '#..', '#..'],
  Q: ['###', '#.#', '#.#', '###', '..#'],
  R: ['###', '#.#', '##.', '#.#', '#.#'],
  S: ['###', '#..', '###', '..#', '###'],
  T: ['###', '.#.', '.#.', '.#.', '.#.'],
  U: ['#.#', '#.#', '#.#', '#.#', '###'],
  V: ['#.#', '#.#', '#.#', '#.#', '.#.'],
  W: ['#.#', '#.#', '###', '###', '#.#'],
  X: ['#.#', '#.#', '.#.', '#.#', '#.#'],
  Y: ['#.#', '#.#', '.#.', '.#.', '.#.'],
  Z: ['###', '..#', '.#.', '#..', '###'],
  '0': ['###', '#.#', '#.#', '#.#', '###'],
  '1': ['.#.', '##.', '.#.', '.#.', '###'],
  '2': ['###', '..#', '###', '#..', '###'],
  '3': ['###', '..#', '###', '..#', '###'],
  '4': ['#.#', '#.#', '###', '..#', '..#'],
  '5': ['###', '#..', '###', '..#', '###'],
  '6': ['###', '#..', '###', '#.#', '###'],
  '7': ['###', '..#', '.#.', '.#.', '.#.'],
  '8': ['###', '#.#', '###', '#.#', '###'],
  '9': ['###', '#.#', '###', '..#', '###'],
  '.': ['...', '...', '...', '...', '.#.'],
  '-': ['...', '...', '###', '...', '...'],
  ':': ['...', '.#.', '...', '.#.', '...'],
  ' ': ['...', '...', '...', '...', '...'],
};

const GLYPH_WIDTH = 3;
const GLYPH_HEIGHT = 5;
const GAP = 1;

interface Pixel {
  readonly id: number;
  readonly x: number;
  readonly y: number;
}

/** Render a short string with the in-repo 3x5 pixel font. */
@Component({
  selector: 'app-pixel-text',
  template: `
    <svg
      class="pixel-text"
      [attr.viewBox]="viewBox()"
      role="img"
      [attr.aria-label]="text()"
      shape-rendering="crispEdges"
    >
      @for (pixel of pixels(); track pixel.id) {
        <rect [attr.x]="pixel.x" [attr.y]="pixel.y" width="1" height="1" fill="currentColor" />
      }
    </svg>
  `,
  styles: `
    .pixel-text {
      display: inline-block;
      height: 1.4em;
      width: auto;
      image-rendering: pixelated;
    }
  `,
})
export class PixelText {
  readonly text = input.required<string>();

  protected readonly pixels = computed<readonly Pixel[]>(() => {
    const pixels: Pixel[] = [];
    let id = 0;
    let cursor = 0;
    for (const character of this.text().toUpperCase()) {
      const glyph = GLYPHS[character] ?? GLYPHS[' '] ?? [];
      for (let row = 0; row < GLYPH_HEIGHT; row += 1) {
        const line = glyph[row] ?? '...';
        for (let column = 0; column < GLYPH_WIDTH; column += 1) {
          if (line[column] === '#') {
            pixels.push({ id, x: cursor + column, y: row });
            id += 1;
          }
        }
      }
      cursor += GLYPH_WIDTH + GAP;
    }
    return pixels;
  });

  protected readonly viewBox = computed(() => {
    const width = Math.max(1, this.text().length * (GLYPH_WIDTH + GAP) - GAP);
    return `0 0 ${width} ${GLYPH_HEIGHT}`;
  });
}
