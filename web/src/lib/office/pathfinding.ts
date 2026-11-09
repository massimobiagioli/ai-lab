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

const ORTHOGONAL = 10;
const DIAGONAL = 14;
const MAX_EXPANSIONS = 4_096;

function key(point: Point): string {
  return `${point.x},${point.y}`;
}

function heuristic(a: Point, b: Point): number {
  const dx = Math.abs(a.x - b.x);
  const dy = Math.abs(a.y - b.y);
  return ORTHOGONAL * (dx + dy) + (DIAGONAL - 2 * ORTHOGONAL) * Math.min(dx, dy);
}

function inside(grid: Grid, point: Point): boolean {
  return point.x >= 0 && point.y >= 0 && point.x < grid.width && point.y < grid.height;
}

/** Binary min-heap ordered by f, then y, then x for determinism. */
class MinHeap {
  private readonly items: Node[] = [];

  get size(): number {
    return this.items.length;
  }

  push(node: Node): void {
    this.items.push(node);
    let index = this.items.length - 1;
    while (index > 0) {
      const parent = (index - 1) >> 1;
      if (this.compare(this.items[index]!, this.items[parent]!)) {
        [this.items[index], this.items[parent]] = [this.items[parent]!, this.items[index]!];
        index = parent;
      } else {
        break;
      }
    }
  }

  pop(): Node | undefined {
    const top = this.items[0];
    const last = this.items.pop();
    if (this.items.length > 0 && last !== undefined) {
      this.items[0] = last;
      let index = 0;
      while (true) {
        const left = index * 2 + 1;
        const right = left + 1;
        let smallest = index;
        if (left < this.items.length && this.compare(this.items[left]!, this.items[smallest]!)) {
          smallest = left;
        }
        if (right < this.items.length && this.compare(this.items[right]!, this.items[smallest]!)) {
          smallest = right;
        }
        if (smallest === index) {
          break;
        }
        [this.items[index], this.items[smallest]] = [this.items[smallest]!, this.items[index]!];
        index = smallest;
      }
    }
    return top;
  }

  private compare(left: Node, right: Node): boolean {
    if (left.f !== right.f) {
      return left.f < right.f;
    }
    if (left.point.y !== right.point.y) {
      return left.point.y < right.point.y;
    }
    return left.point.x < right.point.x;
  }
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
 * Deterministic 8-directional octile A* (orthogonal 10, diagonal 14).
 * Diagonal moves are rejected when either adjacent orthogonal tile is blocked
 * (no corner cutting). Returns the path including start and goal, or null when
 * the goal is blocked, unreachable or the search exceeds 4,096 expansions.
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

  const open = new MinHeap();
  open.push({ point: start, g: 0, f: heuristic(start, goal) });
  const cameFrom = new Map<string, Point>();
  const bestG = new Map<string, number>([[key(start), 0]]);
  const closed = new Set<string>();
  let expansions = 0;

  const neighbours: ReadonlyArray<readonly [number, number, number]> = [
    [1, 0, ORTHOGONAL],
    [-1, 0, ORTHOGONAL],
    [0, 1, ORTHOGONAL],
    [0, -1, ORTHOGONAL],
    [1, 1, DIAGONAL],
    [1, -1, DIAGONAL],
    [-1, 1, DIAGONAL],
    [-1, -1, DIAGONAL],
  ];

  while (open.size > 0) {
    const current = open.pop();
    if (!current) {
      break;
    }
    const currentKey = key(current.point);
    if (closed.has(currentKey)) {
      continue;
    }
    closed.add(currentKey);
    expansions += 1;
    if (expansions > MAX_EXPANSIONS) {
      return null;
    }
    if (current.point.x === goal.x && current.point.y === goal.y) {
      return reconstruct(cameFrom, current.point);
    }
    for (const [dx, dy, cost] of neighbours) {
      const neighbour = { x: current.point.x + dx, y: current.point.y + dy };
      if (!inside(grid, neighbour) || grid.blocked(neighbour)) {
        continue;
      }
      if (dx !== 0 && dy !== 0) {
        const sideA = { x: current.point.x + dx, y: current.point.y };
        const sideB = { x: current.point.x, y: current.point.y + dy };
        if (grid.blocked(sideA) || grid.blocked(sideB)) {
          continue;
        }
      }
      const tentativeG = current.g + cost;
      const neighbourKey = key(neighbour);
      if (bestG.has(neighbourKey) && tentativeG >= (bestG.get(neighbourKey) ?? Infinity)) {
        continue;
      }
      bestG.set(neighbourKey, tentativeG);
      cameFrom.set(neighbourKey, current.point);
      open.push({ point: neighbour, g: tentativeG, f: tentativeG + heuristic(neighbour, goal) });
    }
  }
  return null;
}
