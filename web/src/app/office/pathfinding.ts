export interface Point {
  readonly x: number;
  readonly y: number;
}

export interface Grid {
  readonly width: number;
  readonly height: number;
  readonly blocked: (point: Point) => boolean;
}

interface Node {
  readonly point: Point;
  readonly g: number;
  readonly f: number;
}

function key(point: Point): string {
  return `${point.x},${point.y}`;
}

function heuristic(a: Point, b: Point): number {
  return Math.abs(a.x - b.x) + Math.abs(a.y - b.y);
}

function inside(grid: Grid, point: Point): boolean {
  return point.x >= 0 && point.y >= 0 && point.x < grid.width && point.y < grid.height;
}

function reconstruct(cameFrom: Map<string, Point>, current: Point): Point[] {
  const path: Point[] = [current];
  let cursor = current;
  while (true) {
    const previous = cameFrom.get(key(cursor));
    if (!previous) {
      break;
    }
    path.push(previous);
    cursor = previous;
  }
  return path.reverse();
}

/**
 * Deterministic 4-directional A*. Returns the path including start and goal,
 * or null when the goal is blocked or unreachable.
 */
export function findPath(grid: Grid, start: Point, goal: Point): Point[] | null {
  if (!inside(grid, start) || !inside(grid, goal)) {
    return null;
  }
  if (grid.blocked(start) || grid.blocked(goal)) {
    return null;
  }
  if (start.x === goal.x && start.y === goal.y) {
    return [start];
  }

  const open: Node[] = [{ point: start, g: 0, f: heuristic(start, goal) }];
  const cameFrom = new Map<string, Point>();
  const bestG = new Map<string, number>([[key(start), 0]]);
  const closed = new Set<string>();

  while (open.length > 0) {
    open.sort((left, right) => {
      if (left.f !== right.f) {
        return left.f - right.f;
      }
      if (left.point.y !== right.point.y) {
        return left.point.y - right.point.y;
      }
      return left.point.x - right.point.x;
    });
    const current = open.shift();
    if (!current) {
      break;
    }
    const currentKey = key(current.point);
    if (closed.has(currentKey)) {
      continue;
    }
    closed.add(currentKey);
    if (current.point.x === goal.x && current.point.y === goal.y) {
      return reconstruct(cameFrom, current.point);
    }
    const neighbours: Point[] = [
      { x: current.point.x + 1, y: current.point.y },
      { x: current.point.x - 1, y: current.point.y },
      { x: current.point.x, y: current.point.y + 1 },
      { x: current.point.x, y: current.point.y - 1 },
    ];
    for (const neighbour of neighbours) {
      if (!inside(grid, neighbour) || grid.blocked(neighbour)) {
        continue;
      }
      const tentativeG = current.g + 1;
      const neighbourKey = key(neighbour);
      if (bestG.has(neighbourKey) && tentativeG >= (bestG.get(neighbourKey) ?? Infinity)) {
        continue;
      }
      bestG.set(neighbourKey, tentativeG);
      cameFrom.set(neighbourKey, current.point);
      const f = tentativeG + heuristic(neighbour, goal);
      open.push({ point: neighbour, g: tentativeG, f });
    }
  }
  return null;
}
