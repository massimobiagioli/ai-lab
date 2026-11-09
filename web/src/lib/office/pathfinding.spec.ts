import { findPath, Grid, Point } from './pathfinding';

function grid(blocked: Point[]): Grid {
  return {
    width: 5,
    height: 5,
    blocked: (point) => blocked.some((b) => b.x === point.x && b.y === point.y),
  };
}

describe('findPath', () => {
  it('returns a straight path', () => {
    const path = findPath(grid([]), { x: 0, y: 0 }, { x: 3, y: 0 });
    expect(path).toEqual([
      { x: 0, y: 0 },
      { x: 1, y: 0 },
      { x: 2, y: 0 },
      { x: 3, y: 0 },
    ]);
  });

  it('routes around a wall', () => {
    const wall: Point[] = [
      { x: 2, y: 0 },
      { x: 2, y: 1 },
      { x: 2, y: 2 },
    ];
    const path = findPath(grid(wall), { x: 0, y: 0 }, { x: 4, y: 0 });
    expect(path).not.toBeNull();
    expect(path?.some((point) => point.x === 2 && point.y === 0)).toBe(false);
    expect(path?.[0]).toEqual({ x: 0, y: 0 });
    expect(path?.[path.length - 1]).toEqual({ x: 4, y: 0 });
  });

  it('returns null when the goal is blocked or unreachable', () => {
    expect(findPath(grid([{ x: 4, y: 4 }]), { x: 0, y: 0 }, { x: 4, y: 4 })).toBeNull();
    const wall: Point[] = [];
    for (let y = 0; y < 5; y += 1) {
      wall.push({ x: 2, y });
    }
    expect(findPath(grid(wall), { x: 0, y: 0 }, { x: 4, y: 4 })).toBeNull();
  });

  it('returns the start for an identical endpoint', () => {
    expect(findPath(grid([]), { x: 1, y: 1 }, { x: 1, y: 1 })).toEqual([{ x: 1, y: 1 }]);
  });
});

describe('octile shortcuts', () => {
  it('takes a diagonal when the corner is clear', () => {
    const path = findPath(grid([]), { x: 0, y: 0 }, { x: 2, y: 2 });
    expect(path).toEqual([
      { x: 0, y: 0 },
      { x: 1, y: 1 },
      { x: 2, y: 2 },
    ]);
  });

  it('does not cut a blocked corner', () => {
    const path = findPath(grid([{ x: 1, y: 0 }]), { x: 0, y: 0 }, { x: 2, y: 1 });
    expect(path?.some((point) => point.x === 1 && point.y === 0)).toBe(false);
  });
});
