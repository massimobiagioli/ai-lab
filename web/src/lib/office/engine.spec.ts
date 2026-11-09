import { OfficeEngine } from './engine';
import { DESKS } from './layout';

function engine(): OfficeEngine {
  const desk = DESKS[0];
  if (!desk) {
    throw new Error('missing desk');
  }
  return new OfficeEngine([
    {
      id: 'a',
      name: 'Dev',
      desk,
      state: 'idle',
      position: { x: 1, y: 1 },
      path: [],
      progress: 0,
      elapsedMs: 0,
    },
  ]);
}

describe('OfficeEngine', () => {
  it('routes to a reachable target', () => {
    const office = engine();
    const path = office.moveTo('a', { x: 3, y: 1 });
    expect(path).not.toBeNull();
    expect(path?.length).toBeGreaterThan(1);
  });

  it('advances along the path', () => {
    const office = engine();
    office.moveTo('a', { x: 3, y: 1 });
    office.tick(1_000, false);
    const agent = office.get('a');
    expect(agent?.position.x).toBeGreaterThan(1);
  });

  it('snaps under reduced motion', () => {
    const office = engine();
    office.moveTo('a', { x: 3, y: 1 });
    office.tick(10, true);
    expect(office.get('a')?.position).toEqual({ x: 3, y: 1 });
  });
});

describe('determinism and occupancy', () => {
  it('rejects an occupied target', () => {
    const desk = DESKS[0];
    if (!desk) {
      throw new Error('missing desk');
    }
    const office = new OfficeEngine([
      {
        id: 'a',
        name: 'A',
        desk,
        state: 'idle',
        position: { x: 1, y: 1 },
        path: [],
        progress: 0,
        elapsedMs: 0,
      },
      {
        id: 'b',
        name: 'B',
        desk,
        state: 'idle',
        position: { x: 4, y: 1 },
        path: [],
        progress: 0,
        elapsedMs: 0,
      },
    ]);
    expect(office.moveTo('a', { x: 4, y: 1 })).toBeNull();
  });

  it('produces identical hashes for identical fixed steps', () => {
    const left = engine();
    const right = engine();
    left.moveTo('a', { x: 3, y: 1 });
    right.moveTo('a', { x: 3, y: 1 });
    for (let index = 0; index < 60; index += 1) {
      left.tick(1000 / 60, false);
      right.tick(1000 / 60, false);
    }
    expect(left.frameHash()).toBe(right.frameHash());
  });
});
