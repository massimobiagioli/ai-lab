import type { AgentDomainState, OfficeAnimation } from './agent-state';
import { animationFor, walkingFrame } from './agent-state';
import { OFFICE_GRID, type Desk } from './layout';
import { findPath, type Point } from './pathfinding';

export interface OfficeAgent {
  readonly id: string;
  readonly name: string;
  readonly desk: Desk;
  characterId?: string;
  state: AgentDomainState;
  position: Point;
  path: Point[];
  progress: number;
  elapsedMs: number;
}

export interface RenderPosition {
  readonly x: number;
  readonly y: number;
  readonly frame: number;
  readonly animation: OfficeAnimation;
}

const TILES_PER_SECOND = 6;

/** Framework-agnostic office simulation: agents walk A* paths over a grid. */
export class OfficeEngine {
  private readonly agents: OfficeAgent[];

  constructor(agents: readonly OfficeAgent[]) {
    this.agents = agents.map((agent) => ({ ...agent, path: [], progress: 0, elapsedMs: 0 }));
  }

  get list(): readonly OfficeAgent[] {
    return this.agents;
  }

  get(id: string): OfficeAgent | undefined {
    return this.agents.find((agent) => agent.id === id);
  }

  agentAt(point: Point): OfficeAgent | undefined {
    return this.agents.find(
      (agent) =>
        Math.round(agent.position.x) === point.x && Math.round(agent.position.y) === point.y,
    );
  }

  moveTo(id: string, target: Point): Point[] | null {
    const agent = this.get(id);
    if (!agent) {
      return null;
    }
    const occupied = this.agents.some(
      (other) =>
        other.id !== id &&
        Math.round(other.position.x) === target.x &&
        Math.round(other.position.y) === target.y,
    );
    if (occupied) {
      return null;
    }
    const path = findPath(OFFICE_GRID, agent.position, target);
    if (!path || path.length < 2) {
      return path;
    }
    agent.path = path.slice(1);
    agent.progress = 0;
    agent.elapsedMs = 0;
    return path;
  }

  setState(id: string, state: AgentDomainState): void {
    const agent = this.get(id);
    if (agent) {
      agent.state = state;
    }
  }

  setCharacter(id: string, characterId: string): void {
    const agent = this.get(id);
    if (agent) {
      agent.characterId = characterId;
    }
  }

  tick(deltaMs: number, reducedMotion: boolean): void {
    for (const agent of this.agents) {
      agent.elapsedMs += deltaMs;
      if (agent.path.length === 0) {
        agent.progress = 0;
        continue;
      }
      if (reducedMotion) {
        const last = agent.path[agent.path.length - 1];
        if (last) {
          agent.position = last;
        }
        agent.path = [];
        agent.progress = 0;
        continue;
      }
      agent.progress += (deltaMs / 1000) * TILES_PER_SECOND;
      while (agent.progress >= 1 && agent.path.length > 0) {
        agent.progress -= 1;
        const next = agent.path.shift();
        if (next) {
          agent.position = next;
        }
      }
    }
  }

  /** Stable frame hash for determinism tests. */
  frameHash(): string {
    return this.agents
      .map(
        (agent) =>
          `${agent.id}:${agent.position.x.toFixed(3)},${agent.position.y.toFixed(3)}:${agent.state}`,
      )
      .join('|');
  }

  renderPosition(agent: OfficeAgent): RenderPosition {
    const next = agent.path[0];
    const x = next
      ? agent.position.x + (next.x - agent.position.x) * agent.progress
      : agent.position.x;
    const y = next
      ? agent.position.y + (next.y - agent.position.y) * agent.progress
      : agent.position.y;
    return {
      x,
      y,
      frame: walkingFrame(agent.elapsedMs),
      animation: animationFor(agent.state),
    };
  }
}
