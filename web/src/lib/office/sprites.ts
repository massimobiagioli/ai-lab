import type { OfficeAnimation } from './agent-state';

/** A generated pixel sprite. The atlas is authored in-repo under CC0-1.0. */
export interface Sprite {
  readonly id: string;
  readonly pattern: readonly string[];
  readonly palette: Readonly<Record<string, string>>;
}

/** Manifest metadata for the generated sprite atlas. */
export const SPRITE_MANIFEST = {
  version: 1,
  license: 'CC0-1.0',
  source: 'generated in-repo (no third-party assets)',
  size: 8,
} as const;

export const SPRITES: Readonly<Record<OfficeAnimation, Sprite>> = {
  idle: {
    id: 'agent-idle',
    pattern: [
      '........',
      '..####..',
      '.######.',
      '.#.##.#.',
      '.######.',
      '..####..',
      '..#..#..',
      '.#....#.',
    ],
    palette: { '#': '#f2b84b' },
  },
  type: {
    id: 'agent-type',
    pattern: [
      '........',
      '..####..',
      '.######.',
      '.#.##.#.',
      '.######.',
      '.#.##.#.',
      '..####..',
      '..#..#..',
    ],
    palette: { '#': '#57c785' },
  },
  blocked: {
    id: 'agent-blocked',
    pattern: [
      '........',
      '..####..',
      '.######.',
      '.#.##.#.',
      '.######.',
      '..#..#..',
      '.#.##.#.',
      '#..##..#',
    ],
    palette: { '#': '#e07a5f' },
  },
  sleep: {
    id: 'agent-sleep',
    pattern: [
      '........',
      '..####..',
      '.######.',
      '.#.##.#.',
      '.######.',
      '..####..',
      '........',
      '........',
    ],
    palette: { '#': '#7dd3fc' },
  },
};
