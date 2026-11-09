import { SPRITES, SPRITE_MANIFEST } from './sprites';

describe('sprite atlas', () => {
  it('has a CC0 manifest and a sprite per animation', () => {
    expect(SPRITE_MANIFEST.license).toBe('CC0-1.0');
    expect(Object.keys(SPRITES)).toEqual(['idle', 'type', 'blocked', 'sleep']);
  });

  it('uses 8x8 patterns with a palette', () => {
    for (const sprite of Object.values(SPRITES)) {
      expect(sprite.pattern).toHaveLength(8);
      for (const row of sprite.pattern) {
        expect(row).toHaveLength(8);
      }
      expect(Object.keys(sprite.palette).length).toBeGreaterThan(0);
    }
  });
});
