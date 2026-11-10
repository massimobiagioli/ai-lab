import type { OfficeAnimation } from './agent-state';
import { characterOrDefault, type CharacterProfile } from './characters';
import { drawPixelText, pixelTextWidth } from './font';
import type { OfficeEngine } from './engine';
import { DESKS, FURNITURE, OFFICE_HEIGHT, OFFICE_WIDTH, OFFSET_Y, TILE } from './layout';

const P = {
  ink: '#241A20',
  navy: '#3B2E3A',
  steel: '#8A6A4A',
  periwinkle: '#6B8FBF',
  ice: '#C9E4F2',
  cream: '#F2E9D8',
  amber: '#F2B84B',
  orange: '#E07A3F',
  crimson: '#C0392B',
  teal: '#2A9D8F',
  green: '#57C785',
  magenta: '#C45AA0',
  wood: '#A97142',
  woodLight: '#C98B54',
  woodDark: '#6B4A2E',
  wall: '#D9C7A7',
  wallShade: '#BFA985',
  wallLight: '#F3E7CD',
  floor: '#7C8DA8',
  floorAlt: '#71829C',
  floorLine: '#5D6D86',
  rug: '#3E7C74',
  rugLight: '#5FA79E',
  screen: '#10202E',
  screenGlow: '#7DD3FC',
  metal: '#6E7A96',
  metalDark: '#3E4A66',
  white: '#F8FAFC',
  skin: '#F2C9A0',
  skinShade: '#D9A77B',
  hair: '#2B1B12',
};

const DESK_LABELS = ['TECH LEAD', 'ANALYST', 'DEV', 'QA', 'LOUNGE'] as const;

export function drawOffice(
  context: CanvasRenderingContext2D,
  engine: OfficeEngine,
  selectedId: string | null,
): void {
  const width = OFFICE_WIDTH * TILE;
  const height = OFFICE_HEIGHT * TILE + OFFSET_Y;
  context.clearRect(0, 0, width, height);
  context.imageSmoothingEnabled = false;

  drawWall(context, width);
  drawFloor(context);
  drawRug(context);
  drawMeetingTable(context);
  drawDoor(context);
  drawWindows(context);
  drawClock(context);
  drawWallSign(context);

  for (const object of FURNITURE) {
    drawFurniture(context, object.id, object.point);
  }
  for (const [index, desk] of DESKS.entries()) {
    drawDesk(context, desk.seat, index);
  }

  drawSelectionPath(context, engine, selectedId);
  const ordered = [...engine.list].sort((left, right) => {
    const delta = engine.renderPosition(left).y - engine.renderPosition(right).y;
    return delta !== 0 ? delta : left.id.localeCompare(right.id);
  });
  for (const agent of ordered) {
    const rendered = engine.renderPosition(agent);
    drawAgent(
      context,
      Math.round(rendered.x * TILE) + 2,
      OFFSET_Y + Math.round(rendered.y * TILE) - 14,
      characterOrDefault(agent.characterId),
      rendered.animation,
      rendered.frame,
      agent.id === selectedId,
    );
  }

  drawScanlines(context, width, height);
  drawVignette(context, width, height);
}

function drawWall(context: CanvasRenderingContext2D, width: number): void {
  context.fillStyle = P.ink;
  context.fillRect(0, 0, width, OFFICE_HEIGHT * TILE + OFFSET_Y);
  const top = OFFICE_Y();
  const wallHeight = 3 * TILE;
  context.fillStyle = P.wall;
  context.fillRect(0, top, width, wallHeight);
  // Panel highlights and shadow bands.
  context.fillStyle = P.wallLight;
  context.fillRect(0, top, width, 4);
  context.fillStyle = P.wallShade;
  context.fillRect(0, top + wallHeight - 18, width, 18);
  for (let x = 0; x < width; x += 32) {
    context.fillStyle = P.wallShade;
    context.fillRect(x, top, 1, wallHeight);
    context.fillStyle = P.wallLight;
    context.fillRect(x + 1, top + 2, 30, 1);
  }
  // Wainscoting and baseboard.
  context.fillStyle = P.steel;
  context.fillRect(0, top + wallHeight - 12, width, 9);
  context.fillStyle = P.woodDark;
  context.fillRect(0, top + wallHeight - 3, width, 3);
  context.fillStyle = P.woodLight;
  context.fillRect(0, top + wallHeight - 12, width, 1);
  // Ceiling light.
  context.fillStyle = P.woodDark;
  context.fillRect(width / 2 - 30, top + 2, 60, 4);
  context.fillStyle = P.cream;
  context.fillRect(width / 2 - 26, top + 6, 52, 3);
  context.fillStyle = 'rgba(242,233,216,0.20)';
  context.fillRect(width / 2 - 22, top + 9, 44, 10);
  // Side walls.
  context.fillStyle = P.ink;
  context.fillRect(0, top, 6, wallHeight);
  context.fillRect(width - 6, top, 6, wallHeight);
}

function drawFloor(context: CanvasRenderingContext2D): void {
  const top = OFFICE_Y() + 3 * TILE;
  const height = OFFICE_HEIGHT * TILE - 3 * TILE;
  context.fillStyle = P.floor;
  context.fillRect(0, top, OFFICE_WIDTH * TILE, height);
  for (let y = top; y < top + height; y += 8) {
    context.fillStyle = P.floorAlt;
    context.fillRect(0, y, OFFICE_WIDTH * TILE, 1);
  }
  for (let x = 0; x < OFFICE_WIDTH * TILE; x += 16) {
    context.fillStyle = P.floorLine;
    context.fillRect(x, top, 1, height);
  }
  // Subtle carpet dither and edge shadow.
  context.fillStyle = 'rgba(255,255,255,0.035)';
  for (let y = top + 2; y < top + height; y += 4) {
    for (let x = Math.floor(y / 4) % 8; x < OFFICE_WIDTH * TILE; x += 8) {
      context.fillRect(x, y, 1, 1);
    }
  }
  context.fillStyle = P.ink;
  context.fillRect(0, top, 4, height);
  context.fillRect(OFFICE_WIDTH * TILE - 4, top, 4, height);
}

function drawRug(context: CanvasRenderingContext2D): void {
  const x = 8 * TILE;
  const y = OFFICE_Y() + 4 * TILE;
  context.fillStyle = P.rug;
  context.fillRect(x, y, 8 * TILE, 4 * TILE);
  context.fillStyle = P.rugLight;
  context.fillRect(x, y, 8 * TILE, 2);
  context.fillRect(x, y + 4 * TILE - 2, 8 * TILE, 2);
  context.fillRect(x, y, 2, 4 * TILE);
  context.fillRect(x + 8 * TILE - 2, y, 2, 4 * TILE);
  context.fillStyle = P.cream;
  context.fillRect(x + 4, y + 4, 8 * TILE - 8, 1);
  context.fillRect(x + 4, y + 4 * TILE - 5, 8 * TILE - 8, 1);
  context.fillRect(x + 4, y + 4, 1, 4 * TILE - 8);
  context.fillRect(x + 8 * TILE - 5, y + 4, 1, 4 * TILE - 8);
  context.fillStyle = P.rugLight;
  for (let row = 0; row < 3; row += 1) {
    for (let column = 0; column < 6; column += 1) {
      if ((row + column) % 2 === 0) {
        context.fillRect(x + 14 + column * 16, y + 14 + row * 16, 10, 10);
      }
    }
  }
}

function drawMeetingTable(context: CanvasRenderingContext2D): void {
  const x = 10 * TILE + 3;
  const y = OFFICE_Y() + 4 * TILE + 8;
  context.fillStyle = P.woodDark;
  context.fillRect(x + 2, y + 2, 3 * TILE, 2 * TILE);
  context.fillStyle = P.wood;
  context.fillRect(x, y, 3 * TILE, 2 * TILE);
  context.fillStyle = P.woodLight;
  context.fillRect(x + 2, y + 2, 3 * TILE - 4, 2);
  context.fillStyle = P.cream;
  context.fillRect(x + 8, y + 8, 10, 6);
  context.fillStyle = P.steel;
  context.fillRect(x + 10, y + 10, 6, 1);
  context.fillRect(x + 10, y + 12, 4, 1);
  context.fillStyle = P.ink;
  context.fillRect(x + 3 * TILE - 2, y + 3, 4, 8);
  context.fillRect(x + 3 * TILE - 2, y + 2 * TILE - 8, 4, 8);
}

function drawDoor(context: CanvasRenderingContext2D): void {
  const x = 8;
  const y = OFFICE_Y() + 6 * TILE;
  context.fillStyle = P.woodDark;
  context.fillRect(x - 4, y - 4, 20, 3 * TILE + 8);
  context.fillStyle = P.wood;
  context.fillRect(x, y, 12, 3 * TILE);
  context.fillStyle = P.woodLight;
  context.fillRect(x + 2, y + 2, 8, 2);
  context.fillStyle = P.amber;
  context.fillRect(x + 8, y + TILE, 2, 2);
  const exitWidth = pixelTextWidth('EXIT', 1);
  context.fillStyle = P.ink;
  context.fillRect(x + 2, y - 9, exitWidth + 4, 9);
  drawPixelText(context, 'EXIT', x + 4, y - 7, P.cream, 1);
}

function drawWindows(context: CanvasRenderingContext2D): void {
  for (const [index, start] of [2, 15].entries()) {
    const x = start * TILE;
    const y = OFFICE_Y() + 10;
    context.fillStyle = P.woodDark;
    context.fillRect(x - 3, y - 3, 6 * TILE + 6, 2 * TILE + 6);
    context.fillStyle = P.woodLight;
    context.fillRect(x - 2, y - 2, 6 * TILE + 4, 2 * TILE + 4);
    const sky = context.createLinearGradient(0, y, 0, y + 2 * TILE);
    sky.addColorStop(0, '#2E5C8A');
    sky.addColorStop(0.65, '#6B8FBF');
    sky.addColorStop(1, index === 0 ? '#E8A35A' : '#D98A5A');
    context.fillStyle = sky;
    context.fillRect(x, y, 6 * TILE, 2 * TILE);
    context.fillStyle = P.cream;
    context.fillRect(x + 6, y + 7, 14, 3);
    context.fillRect(x + 22, y + 12, 18, 3);
    context.fillRect(x + 46, y + 6, 12, 2);
    context.fillStyle = '#2A355C';
    for (let i = 0; i < 6; i += 1) {
      const buildingX = x + 6 + i * 14;
      const buildingH = 9 + ((i * 7 + index * 5) % 15);
      context.fillRect(buildingX, y + 2 * TILE - 4 - buildingH, 10, buildingH);
      context.fillStyle = P.amber;
      for (let row = 0; row < 3; row += 1) {
        context.fillRect(buildingX + 2, y + 2 * TILE - 2 - buildingH + row * 4, 1, 1);
        context.fillRect(buildingX + 6, y + 2 * TILE - 2 - buildingH + row * 4, 1, 1);
      }
      context.fillStyle = '#2A355C';
    }
    context.fillStyle = P.woodLight;
    context.fillRect(x + 3 * TILE - 1, y, 2, 2 * TILE);
    context.fillRect(x, y + 2 * TILE - 1, 6 * TILE, 2);
  }
}

function drawClock(context: CanvasRenderingContext2D): void {
  const cx = 12 * TILE;
  const cy = OFFICE_Y() + 18;
  context.fillStyle = P.woodDark;
  context.fillRect(cx - 10, cy - 10, 20, 20);
  context.fillStyle = P.cream;
  context.fillRect(cx - 8, cy - 8, 16, 16);
  context.fillStyle = P.wallShade;
  context.fillRect(cx - 5, cy - 5, 1, 1);
  context.fillRect(cx + 4, cy - 5, 1, 1);
  context.fillRect(cx - 5, cy + 4, 1, 1);
  context.fillRect(cx + 4, cy + 4, 1, 1);
  context.fillStyle = P.ink;
  context.fillRect(cx - 1, cy - 5, 2, 7);
  context.fillRect(cx - 1, cy - 1, 6, 2);
  context.fillStyle = P.crimson;
  context.fillRect(cx - 1, cy - 1, 2, 2);
}

function drawWallSign(context: CanvasRenderingContext2D): void {
  const x = 9 * TILE;
  const y = OFFICE_Y() + 4;
  context.fillStyle = P.woodDark;
  context.fillRect(x - 2, y - 2, 6 * TILE + 4, 15);
  context.fillStyle = P.navy;
  context.fillRect(x, y, 6 * TILE, 11);
  const signWidth = pixelTextWidth('AI LAB', 1);
  drawPixelText(context, 'AI LAB', x + (6 * TILE - signWidth) / 2, y + 2, P.amber, 1);
}

interface Point {
  readonly x: number;
  readonly y: number;
}

function drawFurniture(context: CanvasRenderingContext2D, id: string, point: Point): void {
  const x = point.x * TILE;
  const y = OFFICE_Y() + point.y * TILE;
  switch (id) {
    case 'whiteboard':
      drawWhiteboard(context, x + 2, y - 2);
      break;
    case 'rack':
      drawRack(context, x + 2, y - 4);
      break;
    case 'bookshelf':
      drawBookshelf(context, x + 2, y - 4);
      break;
    case 'cabinet':
      drawCabinet(context, x + 2, y - 2);
      break;
    case 'coffee':
      drawCoffee(context, x + 2, y + 2);
      break;
    case 'plant':
      drawPlant(context, x + 4, y + 4);
      break;
    default:
      break;
  }
}

function drawWhiteboard(context: CanvasRenderingContext2D, x: number, y: number): void {
  context.fillStyle = P.woodDark;
  context.fillRect(x - 2, y - 2, 4 * TILE + 4, 3 * TILE + 4);
  context.fillStyle = P.cream;
  context.fillRect(x, y, 4 * TILE, 3 * TILE);
  context.fillStyle = P.ink;
  context.fillRect(x + 4, y + 4, 4 * TILE - 8, 1);
  const columns = [P.amber, P.teal, P.green, P.magenta];
  for (let column = 0; column < 4; column += 1) {
    context.fillStyle = P.steel;
    context.fillRect(x + 6 + column * 14, y + 8, 10, 1);
    for (let row = 0; row < 3; row += 1) {
      context.fillStyle = columns[column] ?? P.ice;
      context.fillRect(x + 6 + column * 14, y + 12 + row * 8, 10, 6);
    }
  }
  context.fillStyle = P.woodDark;
  context.fillRect(x + 4, y + 3 * TILE, 3, 10);
  context.fillRect(x + 4 * TILE - 7, y + 3 * TILE, 3, 10);
}

function drawRack(context: CanvasRenderingContext2D, x: number, y: number): void {
  context.fillStyle = P.metalDark;
  context.fillRect(x, y, 26, 4 * TILE);
  context.fillStyle = P.metal;
  context.fillRect(x + 2, y + 2, 22, 4 * TILE - 4);
  for (let row = 0; row < 5; row += 1) {
    context.fillStyle = P.screen;
    context.fillRect(x + 4, y + 6 + row * 12, 18, 8);
    context.fillStyle = row % 2 === 0 ? P.green : P.amber;
    context.fillRect(x + 6, y + 8 + row * 12, 2, 2);
    context.fillRect(x + 10, y + 8 + row * 12, 2, 2);
    context.fillStyle = P.ice;
    context.fillRect(x + 15, y + 8 + row * 12, 5, 1);
  }
}

function drawBookshelf(context: CanvasRenderingContext2D, x: number, y: number): void {
  context.fillStyle = P.woodDark;
  context.fillRect(x, y, 3 * TILE, 3 * TILE);
  context.fillStyle = P.wood;
  context.fillRect(x + 2, y + 2, 3 * TILE - 4, 3 * TILE - 4);
  const books = [P.crimson, P.amber, P.teal, P.ice, P.magenta, P.green];
  for (let shelf = 0; shelf < 3; shelf += 1) {
    context.fillStyle = P.woodDark;
    context.fillRect(x + 2, y + 8 + shelf * 14, 3 * TILE - 4, 2);
    for (let book = 0; book < 6; book += 1) {
      context.fillStyle = books[(book + shelf) % books.length] ?? P.ice;
      context.fillRect(x + 5 + book * 6, y + 2 + shelf * 14, 4, 6);
    }
  }
}

function drawCabinet(context: CanvasRenderingContext2D, x: number, y: number): void {
  context.fillStyle = P.metalDark;
  context.fillRect(x, y, 30, 2 * TILE + 2);
  context.fillStyle = P.metal;
  context.fillRect(x + 2, y + 2, 26, 2 * TILE - 2);
  for (let drawer = 0; drawer < 3; drawer += 1) {
    context.fillStyle = P.metalDark;
    context.fillRect(x + 4, y + 6 + drawer * 8, 22, 6);
    context.fillStyle = P.ice;
    context.fillRect(x + 13, y + 8 + drawer * 8, 4, 1);
  }
}

function drawCoffee(context: CanvasRenderingContext2D, x: number, y: number): void {
  context.fillStyle = P.woodDark;
  context.fillRect(x, y + 12, 24, 3);
  context.fillStyle = P.wood;
  context.fillRect(x + 2, y + 10, 20, 2);
  context.fillStyle = P.metalDark;
  context.fillRect(x + 2, y, 14, 12);
  context.fillStyle = P.metal;
  context.fillRect(x + 4, y + 2, 10, 6);
  context.fillStyle = P.crimson;
  context.fillRect(x + 6, y + 4, 2, 2);
  context.fillStyle = P.cream;
  context.fillRect(x + 18, y + 7, 4, 4);
  context.fillStyle = P.ink;
  context.fillRect(x + 19, y + 5, 2, 2);
  context.fillStyle = P.ice;
  context.fillRect(x + 4, y - 4, 1, 3);
  context.fillRect(x + 8, y - 6, 1, 3);
}

function drawPlant(context: CanvasRenderingContext2D, x: number, y: number): void {
  context.fillStyle = P.woodDark;
  context.fillRect(x, y + 12, 12, 10);
  context.fillStyle = P.orange;
  context.fillRect(x + 2, y + 12, 8, 8);
  context.fillStyle = P.green;
  context.fillRect(x + 4, y, 4, 12);
  context.fillRect(x, y + 4, 12, 3);
  context.fillRect(x + 2, y + 1, 2, 5);
  context.fillRect(x + 8, y + 1, 2, 5);
  context.fillStyle = P.teal;
  context.fillRect(x + 6, y + 3, 2, 4);
}

function drawDesk(context: CanvasRenderingContext2D, point: Point, index: number): void {
  const x = point.x * TILE;
  const y = OFFICE_Y() + point.y * TILE;
  context.fillStyle = 'rgba(36,26,32,0.35)';
  context.fillRect(x - 7, y + TILE + 8, 2 * TILE + 14, 5);
  context.fillStyle = P.woodDark;
  context.fillRect(x - 6, y + 6, 2 * TILE + 12, TILE + 4);
  context.fillStyle = P.wood;
  context.fillRect(x - 4, y + 4, 2 * TILE + 8, TILE);
  context.fillStyle = P.woodLight;
  context.fillRect(x - 2, y + 6, 2 * TILE + 4, 1);
  context.fillStyle = P.woodDark;
  context.fillRect(x + 2, y + TILE + 8, 4, 8);
  context.fillRect(x + 2 * TILE - 6, y + TILE + 8, 4, 8);
  context.fillStyle = P.ink;
  context.fillRect(x + 2, y - 4, 14, 10);
  context.fillStyle = P.screen;
  context.fillRect(x + 3, y - 3, 12, 8);
  context.fillStyle = index % 2 === 0 ? P.screenGlow : P.ice;
  context.fillRect(x + 4, y - 2, 10, 1);
  context.fillRect(x + 4, y, 7, 1);
  context.fillStyle = P.metal;
  context.fillRect(x + 16, y + 10, 8, 3);
  context.fillStyle = P.ink;
  context.fillRect(x + 18, y + 13, 4, 2);
  context.fillStyle = P.steel;
  context.fillRect(x + 6, y + 16, 14, 6);
  context.fillStyle = P.navy;
  context.fillRect(x + 8, y + 18, 10, 2);
  context.fillStyle = P.amber;
  context.fillRect(x - 4, y + 2 * TILE - 2, 2 * TILE + 8, 2);
  const label = DESK_LABELS[index] ?? '';
  const labelWidth = pixelTextWidth(label, 1);
  const labelX = x + (2 * TILE - labelWidth) / 2;
  context.fillStyle = P.ink;
  context.fillRect(labelX - 2, y + 2 * TILE + 1, labelWidth + 4, 9);
  drawPixelText(context, label, labelX, y + 2 * TILE + 2, P.cream, 1);
}

function drawAgent(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  character: CharacterProfile,
  animation: OfficeAnimation,
  frame: number,
  selected: boolean,
): void {
  const bob = animation === 'idle' || animation === 'type' ? frame : 0;
  const baseY = y + bob;
  context.fillStyle = 'rgba(11,16,38,0.55)';
  context.fillRect(x + 1, baseY + 22, 14, 3);
  // Legs and shoes.
  context.fillStyle = P.ink;
  context.fillRect(x + 4, baseY + 18, 4, 5);
  context.fillRect(x + 9, baseY + 18, 4, 5);
  context.fillStyle = P.woodDark;
  context.fillRect(x + 3, baseY + 22, 5, 2);
  context.fillRect(x + 9, baseY + 22, 5, 2);
  // Body and collar.
  context.fillStyle = character.outfit;
  context.fillRect(x + 4, baseY + 10, 9, 9);
  context.fillStyle = character.outfitAccent;
  context.fillRect(x + 6, baseY + 10, 5, 2);
  // Arms and hands.
  context.fillStyle = character.skinShade;
  context.fillRect(x + 2, baseY + 11, 2, 7);
  context.fillRect(x + 13, baseY + 11, 2, 7);
  context.fillStyle = character.skin;
  context.fillRect(x + 2, baseY + 17, 2, 2);
  context.fillRect(x + 13, baseY + 17, 2, 2);
  // Head.
  context.fillStyle = character.skin;
  context.fillRect(x + 4, baseY + 2, 9, 8);
  context.fillStyle = character.skinShade;
  context.fillRect(x + 4, baseY + 9, 9, 1);
  context.fillStyle = P.ink;
  context.fillRect(x + 6, baseY + 5, 1, 1);
  context.fillRect(x + 10, baseY + 5, 1, 1);
  context.fillRect(x + 7, baseY + 7, 3, 1);
  drawHair(context, x, baseY, character);
  drawAccessory(context, x, baseY, character, animation);
  if (animation === 'type') {
    context.fillStyle = character.skin;
    context.fillRect(x + 12, baseY + 13, 3, 2);
  }
  if (animation === 'blocked') {
    context.fillStyle = P.crimson;
    context.fillRect(x + 14, baseY - 7, 7, 7);
    context.fillStyle = P.cream;
    context.fillRect(x + 17, baseY - 5, 1, 3);
    context.fillRect(x + 17, baseY - 1, 1, 1);
  }
  if (animation === 'sleep') {
    context.fillStyle = P.ice;
    context.fillRect(x + 14, baseY - 2, 1, 1);
    context.fillRect(x + 16, baseY - 4, 1, 1);
  }
  if (selected) {
    context.strokeStyle = P.amber;
    context.lineWidth = 1;
    context.strokeRect(x - 1, baseY - 2, 17, 29);
  }
}

function drawHair(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  character: CharacterProfile,
): void {
  const hair = character.hair;
  context.fillStyle = hair;
  switch (character.hairStyle) {
    case 'silver':
      context.fillRect(x + 4, y, 9, 3);
      context.fillRect(x + 3, y + 2, 2, 3);
      context.fillRect(x + 12, y + 2, 2, 3);
      break;
    case 'curly':
      context.fillRect(x + 4, y - 1, 9, 3);
      context.fillRect(x + 2, y + 1, 3, 3);
      context.fillRect(x + 12, y + 1, 3, 3);
      context.fillRect(x + 3, y - 2, 3, 3);
      context.fillRect(x + 10, y - 2, 3, 3);
      break;
    case 'buzz':
      context.fillRect(x + 4, y, 9, 2);
      break;
    case 'tall-curly':
      context.fillRect(x + 4, y - 2, 9, 4);
      context.fillRect(x + 2, y, 3, 3);
      context.fillRect(x + 12, y, 3, 3);
      break;
    case 'neat':
      context.fillRect(x + 4, y, 9, 3);
      context.fillStyle = character.skin;
      context.fillRect(x + 8, y, 2, 1);
      break;
    case 'long':
      context.fillRect(x + 4, y - 1, 9, 3);
      context.fillRect(x + 3, y + 2, 2, 9);
      context.fillRect(x + 12, y + 2, 2, 9);
      break;
    case 'auburn':
      context.fillRect(x + 4, y, 9, 3);
      context.fillRect(x + 8, y - 3, 4, 4);
      break;
    case 'cap':
      context.fillStyle = character.outfitAccent;
      context.fillRect(x + 4, y - 2, 9, 3);
      context.fillRect(x + 3, y + 1, 11, 2);
      context.fillStyle = character.hair;
      context.fillRect(x + 3, y + 3, 2, 3);
      context.fillRect(x + 12, y + 3, 2, 3);
      break;
    case 'beanie':
      context.fillStyle = character.outfitAccent;
      context.fillRect(x + 4, y - 2, 9, 5);
      context.fillStyle = character.hair;
      context.fillRect(x + 3, y + 3, 2, 2);
      context.fillRect(x + 12, y + 3, 2, 2);
      break;
    case 'gray':
      context.fillRect(x + 4, y, 9, 3);
      context.fillRect(x + 3, y + 2, 2, 2);
      context.fillRect(x + 12, y + 2, 2, 2);
      break;
  }
}

function drawAccessory(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  character: CharacterProfile,
  animation: OfficeAnimation,
): void {
  switch (character.accessory) {
    case 'glasses':
      context.fillStyle = P.ink;
      context.fillRect(x + 5, y + 5, 3, 1);
      context.fillRect(x + 10, y + 5, 3, 1);
      context.fillRect(x + 8, y + 5, 2, 1);
      break;
    case 'headphones':
      context.fillStyle = P.ink;
      context.fillRect(x + 3, y - 1, 11, 1);
      context.fillRect(x + 2, y + 2, 2, 3);
      context.fillRect(x + 13, y + 2, 2, 3);
      break;
    case 'notebook':
      context.fillStyle = P.cream;
      context.fillRect(x + 12, y + 13, 4, 4);
      context.fillStyle = P.ink;
      context.fillRect(x + 13, y + 14, 2, 1);
      break;
    case 'tablet':
      context.fillStyle = P.screen;
      context.fillRect(x + 12, y + 12, 5, 5);
      context.fillStyle = P.screenGlow;
      context.fillRect(x + 13, y + 13, 3, 3);
      break;
    case 'tie':
      context.fillStyle = character.outfitAccent;
      context.fillRect(x + 8, y + 10, 2, 6);
      break;
    case 'headset':
      context.fillStyle = P.ink;
      context.fillRect(x + 3, y - 1, 11, 1);
      context.fillRect(x + 2, y + 2, 2, 3);
      context.fillRect(x + 12, y + 6, 1, 2);
      context.fillRect(x + 11, y + 8, 2, 1);
      break;
    case 'lanyard':
      context.fillStyle = character.outfitAccent;
      context.fillRect(x + 6, y + 10, 1, 6);
      context.fillRect(x + 10, y + 10, 1, 6);
      context.fillRect(x + 7, y + 15, 4, 3);
      break;
    case 'stylus':
      context.fillStyle = P.cream;
      context.fillRect(x + 14, y + 12, 1, 5);
      break;
    case 'beard':
      context.fillStyle = character.hair;
      context.fillRect(x + 5, y + 8, 7, 3);
      context.fillStyle = character.skin;
      context.fillRect(x + 7, y + 8, 3, 1);
      break;
    case 'mug':
      context.fillStyle = P.cream;
      context.fillRect(x + 13, y + 13, 3, 3);
      context.fillStyle = P.woodDark;
      context.fillRect(x + 16, y + 14, 1, 1);
      break;
  }
  if (animation === 'type') {
    context.fillStyle = character.skin;
    context.fillRect(x + 12, y + 14, 3, 1);
  }
}

function drawSelectionPath(
  context: CanvasRenderingContext2D,
  engine: OfficeEngine,
  selectedId: string | null,
): void {
  if (selectedId === null) {
    return;
  }
  const agent = engine.get(selectedId);
  if (!agent) {
    return;
  }
  for (const step of agent.path) {
    context.fillStyle = P.amber;
    context.fillRect(step.x * TILE + 7, OFFICE_Y() + step.y * TILE + 7, 2, 2);
  }
}

function drawScanlines(context: CanvasRenderingContext2D, width: number, height: number): void {
  context.fillStyle = 'rgba(11,16,38,0.08)';
  for (let y = 0; y < height; y += 2) {
    context.fillRect(0, y, width, 1);
  }
}

function drawVignette(context: CanvasRenderingContext2D, width: number, height: number): void {
  const gradient = context.createRadialGradient(
    width / 2,
    height / 2,
    height / 3,
    width / 2,
    height / 2,
    width / 1.3,
  );
  gradient.addColorStop(0, 'rgba(0,0,0,0)');
  gradient.addColorStop(1, 'rgba(0,0,0,0.38)');
  context.fillStyle = gradient;
  context.fillRect(0, 0, width, height);
}

function OFFICE_Y(): number {
  return OFFSET_Y;
}
