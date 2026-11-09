import type { Point } from './pathfinding';

/** A desk assigned to an agent. */
export interface Desk {
  readonly id: string;
  readonly name: string;
  readonly seat: Point;
}

/** Internal canvas resolution (16-bit pixel office). */
export const CANVAS_WIDTH = 384;
export const CANVAS_HEIGHT = 216;
export const TILE = 16;
export const OFFSET_Y = 4;

export const OFFICE_WIDTH = 24;
export const OFFICE_HEIGHT = 13;

/** Deterministic wall tiles (never walkable). */
const WALLS: readonly Point[] = [
  { x: 8, y: 2 },
  { x: 8, y: 3 },
  { x: 8, y: 4 },
  { x: 15, y: 8 },
  { x: 15, y: 9 },
  { x: 15, y: 10 },
  { x: 11, y: 6 },
  { x: 12, y: 6 },
];

/** Deterministic desks. */
export const DESKS: readonly Desk[] = [
  { id: 'desk-1', name: 'Tech Lead', seat: { x: 3, y: 2 } },
  { id: 'desk-2', name: 'Analyst', seat: { x: 3, y: 8 } },
  { id: 'desk-3', name: 'Dev', seat: { x: 12, y: 10 } },
  { id: 'desk-4', name: 'QA', seat: { x: 19, y: 3 } },
  { id: 'desk-5', name: 'Lounge', seat: { x: 19, y: 10 } },
];

/** Interactive office objects with their drawer content. */
export interface Furniture {
  readonly id: string;
  readonly name: string;
  readonly point: Point;
  readonly panel: string;
  readonly path: string;
}

export const FURNITURE: readonly Furniture[] = [
  {
    id: 'whiteboard',
    name: 'Whiteboard',
    point: { x: 5, y: 4 },
    panel: 'Kanban board',
    path: '/projects/demo/board',
  },
  {
    id: 'rack',
    name: 'Server rack',
    point: { x: 21, y: 2 },
    panel: 'System health',
    path: '/inventory/providers',
  },
  { id: 'bookshelf', name: 'Bookshelf', point: { x: 21, y: 11 }, panel: 'Memory', path: '/memory' },
  { id: 'cabinet', name: 'Cabinet', point: { x: 5, y: 11 }, panel: 'Failed runs', path: '/runs' },
  { id: 'coffee', name: 'Coffee', point: { x: 17, y: 6 }, panel: 'Lounge', path: '/settings' },
  { id: 'plant', name: 'Plant', point: { x: 2, y: 11 }, panel: 'Office', path: '/lab/default' },
  { id: 'door', name: 'Door', point: { x: 0, y: 6 }, panel: 'Exit', path: '/' },
];

const WALL_KEYS = new Set(WALLS.map((point) => `${point.x},${point.y}`));

/** True when a tile cannot be walked through. */
export function isBlocked(point: Point): boolean {
  return (
    point.x < 0 ||
    point.y < 0 ||
    point.x >= OFFICE_WIDTH ||
    point.y >= OFFICE_HEIGHT ||
    WALL_KEYS.has(`${point.x},${point.y}`)
  );
}

/** The shared grid used by the pathfinder. */
export const OFFICE_GRID = {
  width: OFFICE_WIDTH,
  height: OFFICE_HEIGHT,
  blocked: isBlocked,
} as const;
