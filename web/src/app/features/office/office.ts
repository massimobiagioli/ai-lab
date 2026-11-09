import {
  AfterViewInit,
  Component,
  computed,
  ElementRef,
  OnDestroy,
  signal,
  viewChild,
} from '@angular/core';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Card } from '../../shared/ui/card/card';
import { Button } from '../../shared/ui/button/button';
import { StatusBadge, StatusTone } from '../../shared/status-badge/status-badge';
import { OFFICE_HEIGHT, OFFICE_WIDTH, DESKS, isBlocked } from '../../../lib/office/layout';
import { OfficeEngine, type OfficeAgent } from '../../../lib/office/engine';
import type { Point } from '../../../lib/office/pathfinding';
import type { AgentDomainState } from '../../../lib/office/agent-state';

const TILE = 32;
const AGENTS: ReadonlyArray<{
  id: string;
  name: string;
  state: AgentDomainState;
  deskIndex: number;
}> = [
  { id: 'agent-tech-lead', name: 'Tech Lead', state: 'working', deskIndex: 0 },
  { id: 'agent-analyst', name: 'Analyst', state: 'idle', deskIndex: 1 },
  { id: 'agent-dev', name: 'Dev', state: 'working', deskIndex: 2 },
  { id: 'agent-qa', name: 'QA', state: 'blocked', deskIndex: 3 },
  { id: 'agent-lounge', name: 'Lounge', state: 'paused', deskIndex: 4 },
];

@Component({
  selector: 'app-office',
  imports: [PageHeader, Card, Button, StatusBadge],
  template: `
    <app-page-header title="Virtual office" subtitle="Agents, desks and truthful states">
      <app-button (pressed)="moveToLounge()">Move to lounge</app-button>
    </app-page-header>

    <div class="office">
      <app-card title="Floor">
        <canvas
          #canvas
          class="office__canvas"
          [width]="width"
          [height]="height"
          role="img"
          [attr.aria-label]="canvasLabel()"
          (click)="onCanvasClick($event)"
        ></canvas>
        <p class="office__hint">
          Select an agent, then click the floor to walk there. A* routes around the walls.
        </p>
      </app-card>

      <app-card title="Agents">
        <ul class="office__agents">
          @for (agent of engine.list; track agent.id) {
            <li>
              <button
                class="office__agent"
                type="button"
                [class.is-selected]="selectedId() === agent.id"
                (click)="select(agent.id)"
              >
                <span class="office__name">{{ agent.name }}</span>
                <app-status-badge [label]="agent.state" [tone]="tone(agent.state)" />
              </button>
            </li>
          }
        </ul>
        @if (selectedAgent(); as agent) {
          <div class="office__detail">
            <h3>{{ agent.name }}</h3>
            <p>
              State: <strong>{{ agent.state }}</strong>
            </p>
            <p>
              Animation: <strong>{{ animation(agent) }}</strong>
            </p>
            <p>Position: {{ agent.position.x }}, {{ agent.position.y }}</p>
            <div class="office__actions">
              @for (desk of desks; track desk.id) {
                <app-button (pressed)="moveTo(desk.seat)">{{ desk.name }}</app-button>
              }
            </div>
          </div>
        } @else {
          <p class="office__hint">No agent selected.</p>
        }
      </app-card>
    </div>
  `,
  styles: `
    .office {
      display: grid;
      grid-template-columns: minmax(0, 2fr) minmax(240px, 1fr);
      gap: var(--ailab-space-4);
    }
    .office__canvas {
      display: block;
      width: 100%;
      max-width: 640px;
      height: auto;
      image-rendering: pixelated;
      border: 2px solid var(--ailab-color-muted);
      background: var(--ailab-color-surface-sunken);
      cursor: crosshair;
    }
    .office__hint {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
      margin: var(--ailab-space-2) 0 0;
    }
    .office__agents {
      list-style: none;
      margin: 0 0 var(--ailab-space-3);
      padding: 0;
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-1);
    }
    .office__agent {
      display: flex;
      align-items: center;
      justify-content: space-between;
      width: 100%;
      gap: var(--ailab-space-2);
      padding: var(--ailab-space-2);
      border: 2px solid transparent;
      background: var(--ailab-color-surface-sunken);
      color: var(--ailab-color-text);
      font-family: var(--ailab-font-mono);
      cursor: pointer;
    }
    .office__agent.is-selected {
      border-color: var(--ailab-color-accent);
    }
    .office__agent:focus-visible {
      outline: 2px solid var(--ailab-color-accent);
      outline-offset: 2px;
    }
    .office__detail h3 {
      margin: var(--ailab-space-2) 0;
    }
    .office__actions {
      display: flex;
      flex-wrap: wrap;
      gap: var(--ailab-space-1);
      margin-top: var(--ailab-space-2);
    }
    @media (max-width: 900px) {
      .office {
        grid-template-columns: 1fr;
      }
    }
  `,
})
export class Office implements AfterViewInit, OnDestroy {
  protected readonly canvas = viewChild.required<ElementRef<HTMLCanvasElement>>('canvas');
  protected readonly width = OFFICE_WIDTH * TILE;
  protected readonly height = OFFICE_HEIGHT * TILE;
  protected readonly desks = DESKS;
  protected readonly engine = new OfficeEngine(
    AGENTS.map((agent) => {
      const desk = DESKS[agent.deskIndex] ?? DESKS[0];
      if (desk === undefined) {
        throw new Error('office layout has no desks');
      }
      return {
        id: agent.id,
        name: agent.name,
        desk,
        state: agent.state,
        position: desk.seat,
        path: [],
        progress: 0,
        elapsedMs: 0,
      };
    }),
  );
  protected readonly selectedId = signal<string | null>(null);
  private readonly revision = signal(0);
  protected readonly selectedAgent = computed(() => {
    this.revision();
    const id = this.selectedId();
    return id === null ? null : (this.engine.get(id) ?? null);
  });
  protected readonly canvasLabel = computed(
    () => `Office with ${this.engine.list.length} agents and ${DESKS.length} desks`,
  );

  private context: CanvasRenderingContext2D | null = null;
  private frame = 0;
  private last = 0;
  private reducedMotion = false;

  ngAfterViewInit(): void {
    this.context = this.canvas().nativeElement.getContext('2d');
    this.reducedMotion =
      typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
    this.last = performance.now();
    this.loop(this.last);
  }

  ngOnDestroy(): void {
    cancelAnimationFrame(this.frame);
  }

  protected select(id: string): void {
    this.selectedId.set(id);
    this.revision.update((value) => value + 1);
    this.draw();
  }

  protected moveTo(target: Point): void {
    const id = this.selectedId();
    if (id === null) {
      return;
    }
    this.engine.moveTo(id, target);
    this.revision.update((value) => value + 1);
  }

  protected moveToLounge(): void {
    const lounge = DESKS[DESKS.length - 1];
    if (lounge) {
      this.moveTo(lounge.seat);
    }
  }

  protected onCanvasClick(event: MouseEvent): void {
    const canvas = this.canvas().nativeElement;
    const rect = canvas.getBoundingClientRect();
    const x = Math.floor(((event.clientX - rect.left) / rect.width) * OFFICE_WIDTH);
    const y = Math.floor(((event.clientY - rect.top) / rect.height) * OFFICE_HEIGHT);
    const agent = this.engine.agentAt({ x, y });
    if (agent) {
      this.select(agent.id);
      return;
    }
    this.moveTo({ x, y });
  }

  protected tone(state: AgentDomainState): StatusTone {
    switch (state) {
      case 'working':
        return 'success';
      case 'blocked':
        return 'danger';
      case 'idle':
        return 'warning';
      default:
        return 'neutral';
    }
  }

  protected animation(agent: OfficeAgent): string {
    return this.engine.renderPosition(agent).animation;
  }

  private loop = (now: number): void => {
    const delta = Math.min(100, now - this.last);
    this.last = now;
    this.engine.tick(delta, this.reducedMotion);
    this.draw();
    this.frame = requestAnimationFrame(this.loop);
  };

  private draw(): void {
    const context = this.context;
    if (!context) {
      return;
    }
    context.clearRect(0, 0, this.width, this.height);
    for (let y = 0; y < OFFICE_HEIGHT; y += 1) {
      for (let x = 0; x < OFFICE_WIDTH; x += 1) {
        context.fillStyle = (x + y) % 2 === 0 ? '#1e293b' : '#182234';
        context.fillRect(x * TILE, y * TILE, TILE, TILE);
        if (isBlocked({ x, y })) {
          context.fillStyle = '#273449';
          context.fillRect(x * TILE, y * TILE, TILE, TILE);
          context.fillStyle = '#94a3b8';
          context.fillRect(x * TILE, y * TILE, TILE, 3);
        }
      }
    }
    for (const desk of DESKS) {
      context.strokeStyle = '#f2b84b';
      context.lineWidth = 2;
      context.strokeRect(desk.seat.x * TILE + 4, desk.seat.y * TILE + 4, TILE - 8, TILE - 8);
      context.fillStyle = '#94a3b8';
      context.font = '9px monospace';
      context.fillText(desk.name, desk.seat.x * TILE + 2, desk.seat.y * TILE - 2);
    }
    const selected = this.selectedId();
    if (selected !== null) {
      const agent = this.engine.get(selected);
      if (agent) {
        context.fillStyle = 'rgba(242,184,75,0.4)';
        for (const step of agent.path) {
          context.fillRect(step.x * TILE + TILE / 2 - 2, step.y * TILE + TILE / 2 - 2, 4, 4);
        }
      }
    }
    for (const agent of this.engine.list) {
      const rendered = this.engine.renderPosition(agent);
      const bob = rendered.frame % 2 === 0 ? 0 : 2;
      const cx = rendered.x * TILE + 6;
      const cy = rendered.y * TILE + 6 + bob;
      context.fillStyle = this.color(agent.state);
      context.fillRect(cx, cy, TILE - 12, TILE - 12);
      context.fillStyle = '#0f172a';
      context.font = '10px monospace';
      context.fillText(agent.name.slice(0, 2).toUpperCase(), cx + 3, cy + 15);
      if (agent.id === selected) {
        context.strokeStyle = '#ffffff';
        context.lineWidth = 2;
        context.strokeRect(cx - 2, cy - 2, TILE - 8, TILE - 8);
      }
    }
  }

  private color(state: AgentDomainState): string {
    switch (state) {
      case 'working':
        return '#57c785';
      case 'blocked':
        return '#e07a5f';
      case 'idle':
        return '#f2b84b';
      default:
        return '#7dd3fc';
    }
  }
}
