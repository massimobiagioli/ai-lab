import {
  AfterViewInit,
  Component,
  computed,
  effect,
  ElementRef,
  inject,
  OnDestroy,
  signal,
  viewChild,
} from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Card } from '../../shared/ui/card/card';
import { Button } from '../../shared/ui/button/button';
import { StatusBadge, StatusTone } from '../../shared/status-badge/status-badge';
import { Drawer } from '../../shared/ui/drawer/drawer';
import { Tabs } from '../../shared/ui/tabs/tabs';
import { WsService } from '../../core/ws/ws.service';
import {
  CANVAS_HEIGHT,
  CANVAS_WIDTH,
  DESKS,
  OFFICE_WIDTH,
  OFFSET_Y,
  TILE,
} from '../../../lib/office/layout';
import { OfficeEngine, type OfficeAgent } from '../../../lib/office/engine';
import { drawOffice } from '../../../lib/office/art';
import { CHARACTERS, characterOrDefault, type CharacterId } from '../../../lib/office/characters';
import type { Point } from '../../../lib/office/pathfinding';
import type { AgentDomainState } from '../../../lib/office/agent-state';

const AGENTS: ReadonlyArray<{
  id: string;
  name: string;
  state: AgentDomainState;
  deskIndex: number;
  characterId: CharacterId;
}> = [
  {
    id: 'agent-tech-lead',
    name: 'Tech Lead',
    state: 'idle',
    deskIndex: 0,
    characterId: 'veteran_lead',
  },
  {
    id: 'agent-analyst',
    name: 'Analyst',
    state: 'idle',
    deskIndex: 1,
    characterId: 'systems_analyst',
  },
  { id: 'agent-dev', name: 'Dev', state: 'idle', deskIndex: 2, characterId: 'scrappy_dev' },
  { id: 'agent-qa', name: 'QA', state: 'idle', deskIndex: 3, characterId: 'pixel_artisan' },
  {
    id: 'agent-lounge',
    name: 'Lounge',
    state: 'idle',
    deskIndex: 4,
    characterId: 'confident_muse',
  },
];

interface HistoryEntry {
  readonly time: string;
  readonly text: string;
}

interface MemoryEntry {
  readonly title: string;
  readonly snippet: string;
}

const AGENT_CARDS: Readonly<Record<string, string>> = {
  'agent-tech-lead': 'AIL-4 · Release 1.0',
  'agent-analyst': 'AIL-2 · Inventory wizards',
  'agent-dev': 'AIL-1 · Build the Kanban board',
  'agent-qa': 'AIL-4 · Review release candidate',
  'agent-lounge': 'No active card',
};

const AGENT_ACTIVITY: Readonly<Record<string, readonly string[]>> = {
  'agent-tech-lead': [
    'Refined AIL-4 into three acceptance criteria.',
    'Unblocked the Dev on the board filter query.',
    'Queued the QA review for the release candidate.',
  ],
  'agent-analyst': [
    'Updated the SOW traceability matrix.',
    'Wrote acceptance criteria for AIL-2.',
    'Flagged a dependency between AIL-2 and AIL-4.',
  ],
  'agent-dev': [
    'Implemented the keyboard drag and drop flow.',
    'Ran unit tests and fixed the optimistic update.',
    'Opened PR #42 for AIL-1.',
  ],
  'agent-qa': [
    'Verified the six board columns at 390 px.',
    'Rejected one transition with a DoR violation.',
    'Recorded evidence for AIL-4.',
  ],
  'agent-lounge': ['Taking a short break.', 'Coffee machine is ready.'],
};

const AGENT_HISTORY: Readonly<Record<string, readonly HistoryEntry[]>> = {
  'agent-tech-lead': [
    { time: '09:42', text: 'Merged AIL-4 after QA approval.' },
    { time: '09:18', text: 'Assigned AIL-2 to the Analyst.' },
    { time: '08:55', text: 'Created the release candidate card.' },
  ],
  'agent-analyst': [
    { time: '09:31', text: 'Moved AIL-2 to READY.' },
    { time: '09:10', text: 'Added the risk register entry for SCM scopes.' },
    { time: '08:40', text: 'Approved the SOW version 3.' },
  ],
  'agent-dev': [
    { time: '09:38', text: 'Run step implement advanced.' },
    { time: '09:20', text: 'Pushed the keyboard DnD commit.' },
    { time: '08:35', text: 'Picked up AIL-1.' },
  ],
  'agent-qa': [
    { time: '09:35', text: 'Changed state to working.' },
    { time: '09:12', text: 'Ran the responsive board checks.' },
    { time: '08:50', text: 'Approved the inventory drawer change.' },
  ],
  'agent-lounge': [
    { time: '09:00', text: 'Went on a coffee break.' },
    { time: '08:30', text: 'Returned to the lounge desk.' },
  ],
};

const AGENT_MEMORY: Readonly<Record<string, readonly MemoryEntry[]>> = {
  'agent-tech-lead': [
    { title: 'WIP policy', snippet: 'Keep DOING under two cards per agent.' },
    { title: 'Merge rule', snippet: 'QA approval and green checks are mandatory.' },
  ],
  'agent-analyst': [
    { title: 'SOW template', snippet: 'Every FR needs an ID and testable criteria.' },
    { title: 'Traceability', snippet: 'Idea → requirement → plan task → card.' },
  ],
  'agent-dev': [
    { title: 'Board conventions', snippet: 'Columns follow the six-state matrix.' },
    { title: 'Test rule', snippet: 'No optimistic update without a rollback path.' },
  ],
  'agent-qa': [
    { title: 'QA heuristics', snippet: 'Verify acceptance criteria with independent evidence.' },
    { title: 'Responsive gate', snippet: 'Check 1280, 768 and 390 widths.' },
  ],
  'agent-lounge': [{ title: 'Office', snippet: 'Lounge is for idle breaks, not work.' }],
};

@Component({
  selector: 'app-office',
  host: { class: 'office-theme' },
  imports: [PageHeader, Card, Button, StatusBadge, Drawer, Tabs],
  template: `
    <app-page-header title="Virtual office" subtitle="Agents, desks and truthful states">
      <app-button (pressed)="moveToLounge()">Move to lounge</app-button>
    </app-page-header>

    <div class="office">
      <app-card class="office__stage">
        <div class="amiga-window">
          <div class="amiga-window__bar">
            <span class="amiga-window__title">AI-LAB · VIRTUAL OFFICE</span>
            <span class="amiga-window__leds" aria-hidden="true">
              <i class="led led--green"></i>
              <i class="led led--amber"></i>
              <i class="led led--teal"></i>
            </span>
            <span class="amiga-window__buttons" aria-hidden="true"> <i>_</i><i>□</i><i>×</i> </span>
          </div>
          <canvas
            #canvas
            class="office__canvas"
            [width]="width * renderScale"
            [height]="height * renderScale"
            role="img"
            [attr.aria-label]="canvasLabel()"
            (click)="onCanvasClick($event)"
          ></canvas>
          <div class="amiga-window__status">
            <span>{{ engine.list.length }} AGENTS</span>
            <span>{{ fps() }} FPS</span>
            <span>DESKS · ONLINE</span>
          </div>
        </div>
        <p class="office__hint">
          Select an agent, then click the floor to walk there. A* routes around the walls.
        </p>
      </app-card>

      <div class="office__side">
        <app-card title="Agents" aria-live="polite">
          <p class="office__hint">
            Select an agent on the floor or in this list to open the inspector.
          </p>
          <ul class="office__agents">
            @for (agent of engine.list; track agent.id) {
              <li>
                <button
                  class="office__agent"
                  type="button"
                  [attr.data-testid]="'agent-' + agent.id"
                  [class.is-selected]="selectedId() === agent.id"
                  (click)="select(agent.id)"
                >
                  <span class="office__identity">
                    <span class="office__name">{{ agent.name }}</span>
                    <span class="office__character-name">{{ characterName(agent) }}</span>
                  </span>
                  <app-status-badge [label]="agent.state" [tone]="tone(agent.state)" />
                </button>
              </li>
            }
          </ul>
          @if (selectedAgent() === null) {
            <p class="office__hint">No agent selected.</p>
          }
        </app-card>
      </div>
    </div>

    <app-drawer
      side="right"
      label="Agent inspector"
      [open]="inspectorOpen()"
      (closed)="inspectorOpen.set(false)"
    >
      @if (selectedAgent(); as agent) {
        <div class="inspector__head">
          <div>
            <h3>{{ agent.name }}</h3>
            <p class="inspector__character">{{ characterName(agent) }} · {{ agent.desk.name }}</p>
          </div>
          <app-status-badge [label]="agent.state" [tone]="tone(agent.state)" />
        </div>

        <app-tabs label="Agent inspector" [tabs]="inspectorTabs" [(active)]="inspectorTab" />

        @switch (inspectorTab()) {
          @case ('live') {
            <section class="inspector__section">
              <h4>Current card</h4>
              <p>{{ currentCard(agent) }}</p>
              <h4>Activity</h4>
              <ul class="inspector__list">
                @for (activity of activities(agent); track activity) {
                  <li>{{ activity }}</li>
                }
              </ul>
              <p class="inspector__cv">CV: {{ characterName(agent) }} · {{ agent.state }}</p>
              <label class="inspector__character-select">
                <span>Character</span>
                <select [value]="characterId(agent)" (change)="setCharacter($event)">
                  @for (profile of characters; track profile.id) {
                    <option [value]="profile.id" [selected]="profile.id === characterId(agent)">
                      {{ profile.name }}
                    </option>
                  }
                </select>
              </label>
              <p class="inspector__speech">{{ characterSpeech(agent) }}</p>
            </section>
          }
          @case ('history') {
            <section class="inspector__section">
              <h4>History</h4>
              <ul class="inspector__list inspector__list--history">
                @for (entry of history(agent); track entry.time + entry.text) {
                  <li>
                    <time>{{ entry.time }}</time>
                    <span>{{ entry.text }}</span>
                  </li>
                }
              </ul>
            </section>
          }
          @case ('memory') {
            <section class="inspector__section">
              <h4>Memory</h4>
              <ul class="inspector__list">
                @for (note of memory(agent); track note.title) {
                  <li>
                    <strong>{{ note.title }}</strong>
                    <span>{{ note.snippet }}</span>
                  </li>
                }
              </ul>
            </section>
          }
          @default {
            <section class="inspector__section">
              <h4>Controls</h4>
              <div class="inspector__actions">
                @if (agent.state === 'paused') {
                  <app-button variant="primary" (pressed)="resume(agent)">Resume</app-button>
                } @else {
                  <app-button (pressed)="pause(agent)">Pause</app-button>
                }
                <app-button (pressed)="moveTo(agent.desk.seat)">Back to desk</app-button>
              </div>
              <label class="inspector__ask">
                <span>Ask something</span>
                <textarea
                  rows="3"
                  [value]="question()"
                  (input)="onQuestionInput($event)"
                  placeholder="Ask for a status update, a decision or a blocker."
                ></textarea>
              </label>
              <app-button
                variant="primary"
                [disabled]="question().trim().length < 3"
                (pressed)="ask(agent)"
              >
                Send question
              </app-button>
              @if (askStatus() !== '') {
                <p class="inspector__ask-status">{{ askStatus() }}</p>
              }
            </section>
          }
        }
      }
    </app-drawer>
  `,
  styles: `
    :host {
      display: block;
      min-height: 100%;
      background: var(--ailab-color-bg);
    }
    .office {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-4);
    }
    .office__stage {
      display: block;
      padding: 0;
    }
    .amiga-window {
      border: 2px solid #0b1026;
      background: #2e3a6b;
      box-shadow: 4px 4px 0 #0b1026;
    }
    .amiga-window__bar {
      display: flex;
      align-items: center;
      gap: 0.6rem;
      padding: 4px 6px;
      border-bottom: 2px solid #0b1026;
      background: linear-gradient(#5b6fb0, #2e3a6b);
      color: #f2e9d8;
      font-family: var(--ailab-font-mono);
      font-size: 0.68rem;
      font-weight: 700;
      letter-spacing: 0.06em;
    }
    .amiga-window__title {
      flex: 1;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    .amiga-window__leds {
      display: flex;
      gap: 3px;
    }
    .led {
      display: inline-block;
      width: 0.45rem;
      height: 0.45rem;
      border: 1px solid #0b1026;
      background: #9fb3e6;
    }
    .led--green {
      background: #57c785;
    }
    .led--amber {
      background: #f2b84b;
    }
    .led--teal {
      background: #2a9d8f;
    }
    .amiga-window__buttons {
      display: flex;
      gap: 3px;
    }
    .amiga-window__buttons i {
      display: grid;
      place-items: center;
      width: 1rem;
      height: 1rem;
      border: 1px solid #0b1026;
      background: #9fb3e6;
      color: #0b1026;
      font-style: normal;
      font-size: 0.58rem;
      line-height: 1;
    }
    .amiga-window__status {
      display: flex;
      gap: 1rem;
      padding: 3px 6px;
      border-top: 2px solid #0b1026;
      background: #141b3c;
      color: #9fb3e6;
      font-family: var(--ailab-font-mono);
      font-size: 0.62rem;
      letter-spacing: 0.04em;
    }
    .office__canvas {
      display: block;
      max-width: 100%;
      height: auto;
      margin: 0 auto;
      image-rendering: pixelated;
      background: #0b1026;
      cursor: crosshair;
    }
    .office__side {
      display: grid;
      grid-template-columns: 1fr;
      gap: var(--ailab-space-4);
      align-items: start;
    }
    .office__hint {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
      margin: var(--ailab-space-2) 0 0;
    }
    .office__fps {
      color: var(--ailab-color-accent);
      margin-left: var(--ailab-space-2);
    }
    .office__identity {
      display: flex;
      flex-direction: column;
      gap: 0.1rem;
      min-width: 0;
    }
    .office__character-name {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 500;
    }
    .inspector__head {
      display: flex;
      align-items: flex-start;
      justify-content: space-between;
      gap: var(--ailab-space-3);
      margin-bottom: var(--ailab-space-4);
    }
    .inspector__head h3 {
      margin: 0;
      color: var(--ailab-color-text);
      font-size: 1.2rem;
      font-weight: 700;
      letter-spacing: -0.02em;
    }
    .inspector__character {
      margin: 0.2rem 0 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .inspector__section h4 {
      margin: var(--ailab-space-4) 0 var(--ailab-space-2);
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      font-weight: 700;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }
    .inspector__section h4:first-child {
      margin-top: 0;
    }
    .inspector__section p {
      margin: 0;
      color: var(--ailab-color-text-soft);
      font-size: var(--ailab-font-size-md);
    }
    .inspector__list {
      list-style: none;
      margin: 0;
      padding: 0;
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-2);
    }
    .inspector__list li {
      display: flex;
      flex-direction: column;
      gap: 0.15rem;
      padding: 0.6rem 0.7rem;
      border: 1px solid var(--ailab-color-border);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-surface-raised);
      color: var(--ailab-color-text-soft);
      font-size: var(--ailab-font-size-sm);
    }
    .inspector__list--history li {
      flex-direction: row;
      align-items: baseline;
      gap: var(--ailab-space-2);
    }
    .inspector__list time {
      color: var(--ailab-color-muted);
      font-variant-numeric: tabular-nums;
      font-size: var(--ailab-font-size-xs);
      font-weight: 600;
    }
    .inspector__list strong {
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-sm);
    }
    .inspector__list span {
      color: var(--ailab-color-muted);
    }
    .inspector__cv {
      margin-top: var(--ailab-space-4);
      padding: 0.55rem 0.7rem;
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-accent-soft);
      color: var(--ailab-color-accent);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
    }
    .inspector__character-select,
    .inspector__ask {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-1);
      margin-top: var(--ailab-space-4);
      color: var(--ailab-color-text-soft);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
    }
    .inspector__character-select select,
    .inspector__ask textarea {
      padding: 0.6rem 0.7rem;
      border: 1px solid var(--ailab-color-border);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text);
      font: inherit;
      font-weight: 400;
    }
    .inspector__ask textarea {
      resize: vertical;
      min-height: 5rem;
    }
    .inspector__speech {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
      margin: var(--ailab-space-2) 0 0;
    }
    .inspector__actions {
      display: flex;
      flex-wrap: wrap;
      gap: var(--ailab-space-2);
    }
    .inspector__ask-status {
      margin-top: var(--ailab-space-2);
      color: var(--ailab-color-success);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
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
      .office__side {
        grid-template-columns: 1fr;
      }
    }
  `,
})
export class Office implements AfterViewInit, OnDestroy {
  private readonly route = inject(ActivatedRoute);
  private readonly ws = inject(WsService);
  protected readonly canvas = viewChild.required<ElementRef<HTMLCanvasElement>>('canvas');
  protected readonly width = CANVAS_WIDTH;
  protected readonly height = CANVAS_HEIGHT;
  protected readonly renderScale = 2;
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
        characterId: agent.characterId,
        position: desk.seat,
        path: [],
        progress: 0,
        elapsedMs: 0,
      };
    }),
  );
  protected readonly selectedId = signal<string | null>(null);
  protected readonly inspectorOpen = signal(false);
  protected readonly inspectorTab = signal('live');
  protected readonly inspectorTabs = [
    { id: 'live', label: 'Live' },
    { id: 'history', label: 'History' },
    { id: 'memory', label: 'Memory' },
    { id: 'controls', label: 'Controls' },
  ] as const;
  protected readonly question = signal('');
  protected readonly askStatus = signal('');
  private readonly questions = signal<Readonly<Record<string, readonly string[]>>>({});
  protected readonly characters = CHARACTERS;
  protected readonly fps = signal(0);
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
  private smoothedFrameMs = 16;
  private reducedMotion = false;
  private accumulator = 0;
  private readonly onResize = (): void => this.updateScale();

  constructor() {
    effect(() => {
      const event = this.ws.lastEvent();
      if (event?.event !== 'lab.agent.status_changed') {
        return;
      }
      const id = String(event.payload['agent_id'] ?? '');
      const state = String(event.payload['to'] ?? event.payload['state'] ?? '');
      const allowed: readonly AgentDomainState[] = [
        'idle',
        'working',
        'blocked',
        'paused',
        'offline',
      ];
      if (allowed.includes(state as AgentDomainState) && this.engine.get(id)) {
        this.engine.setState(id, state as AgentDomainState);
        this.revision.update((value) => value + 1);
      }
    });
  }

  ngAfterViewInit(): void {
    this.context = this.canvas().nativeElement.getContext('2d');
    this.reducedMotion =
      typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches;
    const linked = this.route.snapshot.queryParamMap.get('agent');
    if (linked !== null && this.engine.get(linked)) {
      this.selectedId.set(linked);
      this.inspectorOpen.set(true);
    }
    const panel = this.route.snapshot.queryParamMap.get('panel');
    if (panel === 'cv' || panel === 'inspector') {
      this.inspectorOpen.set(true);
      this.inspectorTab.set('live');
    }
    if (linked !== null && panel !== 'cv' && panel !== 'inspector') {
      queueMicrotask(() => this.focusAgent(linked));
    }
    this.updateScale();
    window.addEventListener('resize', this.onResize);
    this.last = performance.now();
    this.loop(this.last);
  }

  ngOnDestroy(): void {
    cancelAnimationFrame(this.frame);
    window.removeEventListener('resize', this.onResize);
  }

  private focusAgent(id: string | null): void {
    if (id === null) {
      return;
    }
    document.querySelector<HTMLElement>(`[data-testid="agent-${id}"]`)?.focus();
  }

  private updateScale(): void {
    const canvas = this.canvas().nativeElement;
    const available = canvas.parentElement?.clientWidth ?? CANVAS_WIDTH * this.renderScale;
    const width = CANVAS_WIDTH * this.renderScale;
    const height = CANVAS_HEIGHT * this.renderScale;
    const scale = Math.max(1, Math.min(2, Math.floor(available / width)));
    canvas.style.width = `${scale * width}px`;
    canvas.style.height = `${scale * height}px`;
  }

  protected characterId(agent: OfficeAgent): string {
    return characterOrDefault(agent.characterId).id;
  }

  protected characterName(agent: OfficeAgent): string {
    return characterOrDefault(agent.characterId).name;
  }

  protected characterSpeech(agent: OfficeAgent): string {
    return characterOrDefault(agent.characterId).speech;
  }

  protected setCharacter(event: Event): void {
    const target = event.target;
    const id = this.selectedId();
    if (!(target instanceof HTMLSelectElement) || id === null) {
      return;
    }
    this.engine.setCharacter(id, target.value);
    this.revision.update((value) => value + 1);
    this.draw();
  }

  protected select(id: string): void {
    this.selectedId.set(id);
    this.inspectorOpen.set(true);
    this.inspectorTab.set('live');
    this.question.set('');
    this.askStatus.set('');
    this.revision.update((value) => value + 1);
    this.draw();
  }

  protected pause(agent: OfficeAgent): void {
    this.engine.setState(agent.id, 'paused');
    this.questions.update((value) => ({
      ...value,
      [agent.id]: [...(value[agent.id] ?? []), 'Paused by the operator.'],
    }));
    this.revision.update((value) => value + 1);
    this.draw();
  }

  protected resume(agent: OfficeAgent): void {
    this.engine.setState(agent.id, 'working');
    this.questions.update((value) => ({
      ...value,
      [agent.id]: [...(value[agent.id] ?? []), 'Resumed by the operator.'],
    }));
    this.revision.update((value) => value + 1);
    this.draw();
  }

  protected onQuestionInput(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLTextAreaElement) {
      this.question.set(target.value);
      this.askStatus.set('');
    }
  }

  protected ask(agent: OfficeAgent): void {
    const text = this.question().trim();
    if (text.length < 3) {
      return;
    }
    this.questions.update((value) => ({
      ...value,
      [agent.id]: [...(value[agent.id] ?? []), `You asked: ${text}`],
    }));
    this.question.set('');
    this.askStatus.set(`Question queued for ${agent.name}.`);
    this.revision.update((value) => value + 1);
  }

  protected currentCard(agent: OfficeAgent): string {
    return AGENT_CARDS[agent.id] ?? 'No active card';
  }

  protected activities(agent: OfficeAgent): readonly string[] {
    return [...(AGENT_ACTIVITY[agent.id] ?? []), ...(this.questions()[agent.id] ?? [])];
  }

  protected history(agent: OfficeAgent): readonly HistoryEntry[] {
    return AGENT_HISTORY[agent.id] ?? [];
  }

  protected memory(agent: OfficeAgent): readonly MemoryEntry[] {
    return AGENT_MEMORY[agent.id] ?? [];
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
    const y = Math.floor(
      (((event.clientY - rect.top) / rect.height) * CANVAS_HEIGHT - OFFSET_Y) / TILE,
    );
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
    this.smoothedFrameMs = this.smoothedFrameMs * 0.9 + delta * 0.1;
    if (this.reducedMotion) {
      this.engine.tick(delta, true);
    } else {
      const step = 1000 / 60;
      this.accumulator += delta;
      let steps = 0;
      while (this.accumulator >= step && steps < 3) {
        this.engine.tick(step, false);
        this.accumulator -= step;
        steps += 1;
      }
      if (steps === 3) {
        this.accumulator = 0;
      }
    }
    this.draw();
    this.fps.set(Math.round(1000 / Math.max(1, this.smoothedFrameMs)));
    this.frame = requestAnimationFrame(this.loop);
  };

  private draw(): void {
    const context = this.context;
    if (!context) {
      return;
    }
    context.setTransform(this.renderScale, 0, 0, this.renderScale, 0, 0);
    drawOffice(context, this.engine, this.selectedId());
  }
}
