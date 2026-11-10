import { Component, computed, ElementRef, inject, OnInit, signal, viewChild } from '@angular/core';
import { RouterLink } from '@angular/router';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Spinner } from '../../shared/ui/spinner/spinner';
import { Card } from '../../shared/ui/card/card';
import { Button } from '../../shared/ui/button/button';
import { Icon } from '../../shared/ui/icon/icon';
import { Progress } from '../../shared/ui/progress/progress';
import { EmptyState } from '../../shared/ui/empty-state/empty-state';
import { StatusBadge, type StatusTone } from '../../shared/status-badge/status-badge';
import { ApiClient } from '../../core/api/api-client';
import { DEMO_LAB_ID } from '../../core/demo/demo-data';
import { WsService } from '../../core/ws/ws.service';
import type {
  Agent,
  Card as KanbanCard,
  EventEnvelope,
  LabSummary,
  Problem,
  Provider,
  RunView,
  Sow,
} from '../../domain/models';

const COLUMNS = [
  { id: 'TODO', label: 'To do' },
  { id: 'READY', label: 'Ready' },
  { id: 'DOING', label: 'Doing' },
  { id: 'REVIEW', label: 'Review' },
  { id: 'DONE', label: 'Done' },
  { id: 'ICEBOX', label: 'Icebox' },
] as const;

@Component({
  selector: 'app-dashboard',
  imports: [PageHeader, Card, Button, RouterLink, Spinner, Icon, Progress, EmptyState, StatusBadge],
  template: `
    <app-page-header title="Dashboard" subtitle="Your lab, its work and its signals in one place.">
      @if (demo()) {
        <app-status-badge label="Demo data" tone="warning" />
      }
      <label class="lab-picker">
        <span>Lab</span>
        <select [value]="selectedLabId()" (change)="selectLab($event)" aria-label="Select lab">
          @for (lab of labs(); track lab.id) {
            <option [value]="lab.id">{{ lab.name }}</option>
          }
        </select>
      </label>
      <app-button variant="primary" (pressed)="focusIdea()">New idea</app-button>
    </app-page-header>

    @if (loading()) {
      <app-card><app-spinner label="Loading dashboard" /></app-card>
    } @else if (loadError(); as failure) {
      <app-card title="Unable to load the dashboard">
        <p class="muted">{{ failure.detail ?? failure.title ?? failure.code }}</p>
        <app-button (pressed)="load()">Retry</app-button>
      </app-card>
    } @else {
      <div class="kpis">
        <app-card class="kpi">
          <div class="kpi__top">
            <span class="kpi__icon"><app-icon name="agents" /></span>
            <span class="kpi__label">Active agents</span>
          </div>
          <p class="kpi__value">{{ activeAgents() }}</p>
          <p class="kpi__hint">{{ agents().length }} in the roster</p>
        </app-card>

        <app-card class="kpi kpi--wide">
          <div class="kpi__top">
            <span class="kpi__icon kpi__icon--violet"><app-icon name="board" /></span>
            <span class="kpi__label">Cards by column</span>
            <span class="kpi__total">{{ cards().length }} total</span>
          </div>
          <div class="columns">
            @for (column of columnCounts(); track column.id) {
              <div class="column-stat">
                <span class="column-stat__count">{{ column.count }}</span>
                <span class="column-stat__label">{{ column.label }}</span>
              </div>
            }
          </div>
        </app-card>

        <app-card class="kpi">
          <div class="kpi__top">
            <span class="kpi__icon kpi__icon--green"><app-icon name="providers" /></span>
            <span class="kpi__label">Spend today</span>
          </div>
          <p class="kpi__value">{{ spendToday() }}</p>
          <p class="kpi__hint">Model usage across tracked runs</p>
        </app-card>

        <app-card class="kpi">
          <div class="kpi__top">
            <span class="kpi__icon kpi__icon--amber"><app-icon name="runs" /></span>
            <span class="kpi__label">Success rate</span>
          </div>
          <p class="kpi__value">{{ successRate() }}%</p>
          <p class="kpi__hint">{{ succeeded() }} of {{ runs().length }} runs succeeded</p>
        </app-card>
      </div>

      <div class="layout">
        <div class="layout__main">
          <app-card title="New idea">
            <p class="muted intro">
              Describe the outcome. The Tech Lead turns it into estimable cards on the board.
            </p>
            <textarea
              #ideaInput
              class="idea-input"
              rows="3"
              placeholder="Example: Add a customer feedback digest with a weekly summary and a review queue."
              [value]="idea()"
              (input)="onIdeaInput($event)"
              aria-label="New idea"
            ></textarea>
            <div class="idea-actions">
              <span class="idea-hint">{{ ideaHint() }}</span>
              <app-button
                variant="primary"
                [disabled]="idea().trim().length < 10"
                [loading]="ideaState() === 'sending'"
                (pressed)="submitIdea()"
              >
                Send to Tech Lead
              </app-button>
            </div>
            @if (ideaMessage() !== '') {
              <p class="idea-message" [class.is-error]="ideaState() === 'error'">
                {{ ideaMessage() }}
              </p>
            }
          </app-card>

          <app-card title="Activity">
            @if (events().length === 0) {
              <app-empty-state
                title="No activity yet"
                message="Events appear here as soon as the team starts moving cards."
              />
            } @else {
              <ol class="feed">
                @for (event of recentEvents(); track event.event_id) {
                  <li class="feed__item">
                    <span class="feed__dot" [class]="'feed__dot feed__dot--' + eventTone(event)">
                    </span>
                    <div class="feed__body">
                      <p class="feed__title">{{ eventTitle(event) }}</p>
                      <p class="feed__detail">{{ eventDetail(event) }}</p>
                    </div>
                    <time class="feed__time">{{ formatTime(event.occurred_at) }}</time>
                  </li>
                }
              </ol>
              <a class="text-link" routerLink="/runs">Open runs and trace</a>
            }
          </app-card>

          <app-card title="Recent SOW">
            @if (sow(); as document) {
              <div class="sow">
                <div>
                  <p class="sow__title">{{ document.id }}</p>
                  <p class="muted">
                    Version {{ document.version }} · {{ document.tasks.length }} tasks ·
                    {{ document.sections.length }} sections
                  </p>
                </div>
                <app-status-badge [label]="document.status" [tone]="sowTone(document.status)" />
              </div>
              <a class="text-link" routerLink="/projects/demo/sow">Open SOW workspace</a>
            } @else {
              <app-empty-state
                title="No SOW yet"
                message="An approved SOW appears here after the Analyst drafts it."
              />
            }
          </app-card>
        </div>

        <aside class="layout__side">
          <app-card title="Live office">
            <a class="office-link" routerLink="/lab/default" aria-label="Open the virtual office">
              <div class="mini-room" aria-hidden="true">
                <span class="mini-room__floor"></span>
                <span class="mini-room__desk mini-room__desk--one"></span>
                <span class="mini-room__desk mini-room__desk--two"></span>
                <span class="mini-room__desk mini-room__desk--three"></span>
                <span class="mini-room__desk mini-room__desk--four"></span>
                <span class="mini-room__agent mini-room__agent--one"></span>
                <span class="mini-room__agent mini-room__agent--two"></span>
                <span class="mini-room__agent mini-room__agent--three"></span>
                <span class="mini-room__agent mini-room__agent--four"></span>
                <span class="mini-room__plant"></span>
              </div>
              <span class="office-link__caption">
                <span>See the team at work</span>
                <app-status-badge label="Live" tone="success" />
              </span>
            </a>
            <a class="text-link" routerLink="/lab/default">Open the virtual office</a>
          </app-card>

          <app-card title="System health">
            <div class="status-row">
              <span>Realtime channel</span>
              @if (demo()) {
                <app-status-badge label="Demo mode" tone="warning" />
              } @else {
                <app-status-badge [label]="connection()" [tone]="connectionTone()" />
              }
            </div>
            <div class="status-row">
              <span>Provider</span>
              <app-status-badge [label]="providerLabel()" [tone]="providerTone()" />
            </div>
            <div class="status-row">
              <span>Workspace</span>
              <strong>Local</strong>
            </div>
            <a class="text-link" routerLink="/settings">Open health and settings</a>
          </app-card>

          @if (!onboardingDismissed()) {
            <app-card title="Getting started">
              <div class="onboarding__progress">
                <p class="onboarding__count">
                  {{ doneSteps().size }} of {{ onboarding.length }} steps
                </p>
                <app-progress [value]="onboardingProgress()" label="Onboarding progress" />
              </div>
              <ol class="onboarding">
                @for (step of onboarding; track step.id) {
                  <li class="onboarding__step" [class.is-done]="doneSteps().has(step.id)">
                    <span class="onboarding__check" aria-hidden="true">
                      {{ doneSteps().has(step.id) ? '✓' : '' }}
                    </span>
                    <div class="onboarding__body">
                      <a [routerLink]="step.path">{{ step.label }}</a>
                      <p>{{ step.description }}</p>
                    </div>
                  </li>
                }
              </ol>
              <div class="onboarding__footer">
                <app-button (pressed)="onboardingDismissed.set(true)">Dismiss</app-button>
              </div>
            </app-card>
          }
        </aside>
      </div>
    }
  `,
  styles: `
    .lab-picker {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-2);
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
    }
    .lab-picker select {
      min-height: 2.375rem;
      padding: 0.45rem 0.75rem;
      border: 1px solid var(--ailab-color-border);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text);
      font: inherit;
    }
    .kpis {
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: var(--ailab-space-4);
      margin-bottom: var(--ailab-space-6);
    }
    .kpi {
      display: block;
    }
    .kpi__top {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-2);
      margin-bottom: var(--ailab-space-3);
    }
    .kpi__icon {
      display: grid;
      place-items: center;
      width: 2.1rem;
      height: 2.1rem;
      border-radius: var(--ailab-radius-sm);
      background: var(--ailab-color-accent-soft);
      color: var(--ailab-color-accent);
    }
    .kpi__icon--violet {
      background: #f3e8ff;
      color: #7e22ce;
    }
    .kpi__icon--green {
      background: var(--ailab-color-success-soft);
      color: var(--ailab-color-success);
    }
    .kpi__icon--amber {
      background: var(--ailab-color-warning-soft);
      color: var(--ailab-color-warning);
    }
    .kpi__label {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
    }
    .kpi__total {
      margin-left: auto;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
    }
    .kpi__value {
      margin: 0;
      color: var(--ailab-color-text);
      font-size: clamp(1.7rem, 3vw, 2.15rem);
      font-weight: 750;
      letter-spacing: -0.04em;
      line-height: 1;
    }
    .kpi__hint {
      margin: var(--ailab-space-2) 0 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .columns {
      display: grid;
      grid-template-columns: repeat(6, minmax(0, 1fr));
      gap: var(--ailab-space-1);
      margin-top: var(--ailab-space-2);
    }
    .column-stat {
      display: flex;
      flex-direction: column;
      gap: 0.1rem;
      padding: 0.4rem 0.35rem;
      border-radius: var(--ailab-radius-sm);
      background: var(--ailab-color-surface-sunken);
      text-align: center;
    }
    .column-stat__count {
      color: var(--ailab-color-text);
      font-weight: 700;
      font-size: var(--ailab-font-size-md);
    }
    .column-stat__label {
      color: var(--ailab-color-muted);
      font-size: 0.6rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
    .layout {
      display: grid;
      grid-template-columns: minmax(0, 1.65fr) minmax(280px, 1fr);
      gap: var(--ailab-space-6);
      align-items: start;
    }
    .layout__main,
    .layout__side {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-6);
      min-width: 0;
    }
    .intro {
      margin-top: 0;
    }
    .idea-input {
      width: 100%;
      resize: vertical;
      min-height: 6rem;
      padding: 0.8rem 0.9rem;
      border: 1px solid var(--ailab-color-border);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text);
      font: inherit;
      line-height: 1.5;
    }
    .idea-input:focus-visible {
      outline: none;
      border-color: var(--ailab-color-accent);
      box-shadow: 0 0 0 3px var(--ailab-color-accent-soft);
    }
    .idea-actions {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-3);
      margin-top: var(--ailab-space-3);
    }
    .idea-hint {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
    }
    .idea-message {
      margin: var(--ailab-space-3) 0 0;
      color: var(--ailab-color-success);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
    }
    .idea-message.is-error {
      color: var(--ailab-color-danger);
    }
    .feed {
      list-style: none;
      margin: 0;
      padding: 0;
      display: flex;
      flex-direction: column;
    }
    .feed__item {
      display: flex;
      align-items: flex-start;
      gap: var(--ailab-space-3);
      padding: 0.75rem 0;
      border-bottom: 1px solid var(--ailab-color-border);
    }
    .feed__item:last-child {
      border-bottom: 0;
    }
    .feed__dot {
      width: 0.55rem;
      height: 0.55rem;
      margin-top: 0.45rem;
      border-radius: 50%;
      background: var(--ailab-color-muted);
      flex: none;
    }
    .feed__dot--success {
      background: var(--ailab-color-success);
    }
    .feed__dot--warning {
      background: var(--ailab-color-warning);
    }
    .feed__dot--info {
      background: var(--ailab-color-info);
    }
    .feed__body {
      min-width: 0;
      flex: 1;
    }
    .feed__title {
      margin: 0;
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-md);
      font-weight: 600;
    }
    .feed__detail {
      margin: 0.1rem 0 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .feed__time {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
      white-space: nowrap;
    }
    .sow {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-3);
    }
    .sow__title {
      margin: 0;
      color: var(--ailab-color-text);
      font-weight: 650;
    }
    .text-link {
      display: inline-block;
      margin-top: var(--ailab-space-3);
      color: var(--ailab-color-accent);
      font-size: var(--ailab-font-size-sm);
      font-weight: 650;
      text-decoration: none;
    }
    .office-link {
      display: block;
      color: inherit;
      text-decoration: none;
    }
    .mini-room {
      position: relative;
      height: 8.5rem;
      overflow: hidden;
      border-radius: var(--ailab-radius-md);
      background:
        repeating-linear-gradient(0deg, transparent 0 15px, rgb(15 23 42 / 5%) 15px 16px),
        repeating-linear-gradient(90deg, transparent 0 15px, rgb(15 23 42 / 5%) 15px 16px), #dbe3ef;
    }
    .mini-room__floor {
      position: absolute;
      inset: auto 0 0;
      height: 38%;
      background: #c7d2e3;
    }
    .mini-room__desk {
      position: absolute;
      width: 2.2rem;
      height: 1.1rem;
      border-radius: 3px;
      background: #64748b;
      box-shadow: inset 0 -4px 0 rgb(15 23 42 / 22%);
    }
    .mini-room__desk--one {
      top: 2rem;
      left: 1.1rem;
    }
    .mini-room__desk--two {
      top: 2rem;
      left: 4.4rem;
    }
    .mini-room__desk--three {
      top: 2rem;
      left: 7.7rem;
    }
    .mini-room__desk--four {
      top: 4.6rem;
      left: 2.8rem;
    }
    .mini-room__agent {
      position: absolute;
      width: 0.9rem;
      height: 1.15rem;
      border-radius: 2px;
      background: #4f46e5;
      box-shadow: inset 0 -4px 0 rgb(15 23 42 / 20%);
    }
    .mini-room__agent--one {
      top: 0.8rem;
      left: 1.75rem;
    }
    .mini-room__agent--two {
      top: 0.8rem;
      left: 5.05rem;
      background: #0ea5e9;
    }
    .mini-room__agent--three {
      top: 0.8rem;
      left: 8.35rem;
      background: #16a34a;
    }
    .mini-room__agent--four {
      top: 3.4rem;
      left: 3.45rem;
      background: #d97706;
    }
    .mini-room__plant {
      position: absolute;
      right: 1rem;
      bottom: 0.7rem;
      width: 1.1rem;
      height: 1.6rem;
      border-radius: 50% 50% 20% 20%;
      background: #2a9d8f;
      box-shadow: inset 0 -6px 0 rgb(15 23 42 / 18%);
    }
    .office-link__caption {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-2);
      margin-top: var(--ailab-space-3);
      color: var(--ailab-color-text);
      font-weight: 650;
    }
    .status-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: var(--ailab-space-3);
      padding: 0.55rem 0;
      border-bottom: 1px solid var(--ailab-color-border);
      color: var(--ailab-color-text-soft);
      font-size: var(--ailab-font-size-md);
    }
    .status-row:last-of-type {
      border-bottom: 0;
    }
    .onboarding__progress {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-3);
      margin-bottom: var(--ailab-space-3);
    }
    .onboarding__progress app-progress {
      flex: 1;
      min-width: 4rem;
    }
    .onboarding__count {
      margin: 0;
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-sm);
      font-weight: 650;
      white-space: nowrap;
    }
    .onboarding {
      list-style: none;
      margin: 0;
      padding: 0;
      display: flex;
      flex-direction: column;
    }
    .onboarding__step {
      display: flex;
      align-items: flex-start;
      gap: var(--ailab-space-2);
      padding: 0.5rem 0;
      border-bottom: 1px solid var(--ailab-color-border);
    }
    .onboarding__step:last-child {
      border-bottom: 0;
    }
    .onboarding__check {
      display: grid;
      place-items: center;
      width: 1.2rem;
      height: 1.2rem;
      margin-top: 0.1rem;
      border: 1px solid var(--ailab-color-border-strong);
      border-radius: 50%;
      color: transparent;
      font-size: 0.65rem;
      font-weight: 700;
      flex: none;
    }
    .onboarding__step.is-done .onboarding__check {
      border-color: var(--ailab-color-success);
      background: var(--ailab-color-success-soft);
      color: var(--ailab-color-success);
    }
    .onboarding__body {
      min-width: 0;
    }
    .onboarding__body a {
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
      text-decoration: none;
    }
    .onboarding__body p {
      margin: 0.1rem 0 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
    }
    .onboarding__footer {
      margin-top: var(--ailab-space-3);
    }
    .muted {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    @media (max-width: 1150px) {
      .kpis {
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }
      .layout {
        grid-template-columns: 1fr;
      }
      .layout__side {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        align-items: start;
      }
    }
    @media (max-width: 760px) {
      .kpis {
        grid-template-columns: 1fr;
      }
      .layout__side {
        display: flex;
      }
      .columns {
        grid-template-columns: repeat(3, minmax(0, 1fr));
      }
      .idea-actions {
        align-items: flex-start;
        flex-direction: column;
      }
      .feed__time {
        display: none;
      }
    }
  `,
})
export class Dashboard implements OnInit {
  private readonly api = inject(ApiClient);
  private readonly ws = inject(WsService);
  private readonly ideaInput = viewChild<ElementRef<HTMLTextAreaElement>>('ideaInput');

  protected readonly loading = signal(true);
  protected readonly loadError = signal<Problem | null>(null);
  protected readonly labs = signal<readonly LabSummary[]>([]);
  protected readonly selectedLabId = signal(DEMO_LAB_ID);
  protected readonly agents = signal<readonly Agent[]>([]);
  protected readonly cards = signal<readonly KanbanCard[]>([]);
  protected readonly events = signal<readonly EventEnvelope[]>([]);
  protected readonly runs = signal<readonly RunView[]>([]);
  protected readonly providers = signal<readonly Provider[]>([]);
  protected readonly sow = signal<Sow | null>(null);
  protected readonly idea = signal('');
  protected readonly ideaState = signal<'idle' | 'sending' | 'sent' | 'error'>('idle');
  protected readonly ideaMessage = signal('');
  protected readonly onboardingDismissed = signal(false);
  protected readonly doneSteps = signal<ReadonlySet<string>>(new Set(['provider']));
  protected readonly connection = this.ws.state;
  protected readonly demo = this.api.usingDemo;
  protected readonly onboarding = [
    {
      id: 'provider',
      label: 'Connect a provider',
      description: 'Add DeepSeek and keep the key write-only.',
      path: '/inventory/providers',
    },
    {
      id: 'key',
      label: 'Store the provider key',
      description: 'Load the live key only for live runs.',
      path: '/inventory/providers',
    },
    {
      id: 'lab',
      label: 'Create the lab',
      description: 'Set autonomy and the monthly budget.',
      path: '/settings',
    },
    {
      id: 'tech-lead',
      label: 'Create the Tech Lead',
      description: 'Every lab has one active Tech Lead.',
      path: '/inventory/agents',
    },
    {
      id: 'github',
      label: 'Connect GitHub',
      description: 'Bind the repository and its credential.',
      path: '/inventory/projects',
    },
    {
      id: 'project',
      label: 'Create a project',
      description: 'Pick a key prefix and default branch.',
      path: '/inventory/projects',
    },
    {
      id: 'idea',
      label: 'Submit the first idea',
      description: 'The Tech Lead turns it into ready cards.',
      path: '/',
    },
  ] as const;

  protected readonly activeAgents = computed(
    () => this.agents().filter((agent) => agent.status !== 'offline').length,
  );
  protected readonly columnCounts = computed(() => {
    const cards = this.cards();
    return COLUMNS.map((column) => ({
      ...column,
      count: cards.filter((card) => card.column === column.id).length,
    }));
  });
  protected readonly recentEvents = computed(() => this.events().slice(0, 8));
  protected readonly succeeded = computed(
    () => this.runs().filter((run) => run.state === 'succeeded').length,
  );
  protected readonly successRate = computed(() => {
    const runs = this.runs();
    if (runs.length === 0) {
      return 0;
    }
    return Math.round((this.succeeded() / runs.length) * 100);
  });
  protected readonly spendToday = computed(() => {
    const today = new Date().toISOString().slice(0, 10);
    const micros = this.runs()
      .filter((run) => run.created_at.slice(0, 10) === today)
      .reduce((total, run) => total + run.cost_microusd, 0);
    return this.formatMoney(micros);
  });
  protected readonly onboardingProgress = computed(() =>
    Math.round((this.doneSteps().size / this.onboarding.length) * 100),
  );
  protected readonly providerLabel = computed(() => {
    const providers = this.providers();
    if (providers.length === 0) {
      return 'Not configured';
    }
    return providers.some((provider) => provider.status === 'active')
      ? 'Active'
      : 'Needs attention';
  });
  protected readonly providerTone = computed<StatusTone>(() => {
    const providers = this.providers();
    if (providers.length === 0) {
      return 'warning';
    }
    return providers.some((provider) => provider.status === 'active') ? 'success' : 'danger';
  });

  ngOnInit(): void {
    void this.load();
  }

  protected async load(): Promise<void> {
    this.loading.set(true);
    this.loadError.set(null);
    const labId = this.selectedLabId();
    const results = await Promise.allSettled([
      this.api.listLabs(),
      this.api.listAgents(),
      this.api.getBoard('demo'),
      this.api.listRuns(labId),
      this.api.listEvents(labId, 0, 50),
      this.api.getSow('demo'),
      this.api.listProviders(),
    ]);
    const [labs, agents, cards, runs, events, sow, providers] = results;
    if (labs.status === 'fulfilled') {
      this.labs.set(labs.value);
      if (labs.value.length > 0 && !labs.value.some((lab) => lab.id === this.selectedLabId())) {
        this.selectedLabId.set(labs.value[0]?.id ?? DEMO_LAB_ID);
      }
    }
    if (agents.status === 'fulfilled') {
      this.agents.set(agents.value);
    }
    if (cards.status === 'fulfilled') {
      this.cards.set(cards.value);
    }
    if (runs.status === 'fulfilled') {
      this.runs.set(runs.value.data);
    }
    if (events.status === 'fulfilled') {
      this.events.set(events.value.data);
    }
    if (sow.status === 'fulfilled') {
      this.sow.set(sow.value);
    }
    if (providers.status === 'fulfilled') {
      this.providers.set(providers.value);
    }
    const failure = results.find(
      (result): result is PromiseRejectedResult => result.status === 'rejected',
    );
    if (failure !== undefined && this.labs().length === 0 && this.runs().length === 0) {
      this.loadError.set(this.toProblem(failure.reason));
    }
    this.loading.set(false);
  }

  protected selectLab(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLSelectElement) {
      this.selectedLabId.set(target.value);
      void this.load();
    }
  }

  protected onIdeaInput(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLTextAreaElement) {
      this.idea.set(target.value);
      if (this.ideaState() !== 'sending') {
        this.ideaState.set('idle');
        this.ideaMessage.set('');
      }
    }
  }

  protected focusIdea(): void {
    this.ideaInput()?.nativeElement.focus();
    this.ideaInput()?.nativeElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }

  protected async submitIdea(): Promise<void> {
    const text = this.idea().trim();
    if (text.length < 10) {
      return;
    }
    this.ideaState.set('sending');
    this.ideaMessage.set('');
    try {
      await this.api.submitIdea(text);
      this.ideaState.set('sent');
      this.ideaMessage.set('Idea queued. The Tech Lead will shape it into cards.');
      this.idea.set('');
    } catch (error) {
      this.ideaState.set('error');
      const problem = this.toProblem(error);
      this.ideaMessage.set(problem.detail ?? problem.title ?? 'The idea could not be queued.');
    }
  }

  protected markDone(id: string): void {
    this.doneSteps.update((current) => new Set([...current, id]));
  }

  protected ideaHint(): string {
    const length = this.idea().trim().length;
    return length < 10 ? 'At least 10 characters.' : `${length} characters ready to send.`;
  }

  protected eventTitle(event: EventEnvelope): string {
    switch (event.event) {
      case 'delivery.run.finished':
        return 'Run finished';
      case 'delivery.run.step':
        return 'Run step completed';
      case 'delivery.run.queued':
        return 'Run queued';
      case 'kanban.card.moved':
        return 'Card moved';
      case 'lab.agent.status_changed':
        return 'Agent state changed';
      case 'memory.document.indexed':
        return 'Memory note indexed';
      default:
        return event.event;
    }
  }

  protected eventDetail(event: EventEnvelope): string {
    const payload = event.payload;
    switch (event.event) {
      case 'delivery.run.finished':
        return `Run ${String(payload['run_id'] ?? '')} is ${String(payload['state'] ?? 'done')}.`;
      case 'delivery.run.step':
        return `Stage ${String(payload['stage'] ?? 'unknown')} advanced.`;
      case 'delivery.run.queued':
        return `Card ${String(payload['card_key'] ?? '')} entered the queue.`;
      case 'kanban.card.moved':
        return `${String(payload['card_key'] ?? 'Card')} moved to ${String(payload['to'] ?? '')}.`;
      case 'lab.agent.status_changed':
        return `${String(payload['agent_id'] ?? 'Agent')} is now ${String(payload['status'] ?? '')}.`;
      case 'memory.document.indexed':
        return `${String(payload['title'] ?? 'Document')} indexed in ${String(payload['scope'] ?? 'memory')}.`;
      default:
        return event.event;
    }
  }

  protected eventTone(event: EventEnvelope): 'success' | 'warning' | 'info' {
    if (event.event === 'delivery.run.finished') {
      return 'success';
    }
    if (event.event === 'lab.agent.status_changed') {
      return 'warning';
    }
    return 'info';
  }

  protected formatTime(value: string): string {
    return new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  protected connectionTone(): StatusTone {
    switch (this.connection()) {
      case 'connected':
        return 'success';
      case 'reconnecting':
        return 'warning';
      default:
        return 'danger';
    }
  }

  protected sowTone(status: string): StatusTone {
    switch (status) {
      case 'approved':
        return 'success';
      case 'rejected':
        return 'danger';
      default:
        return 'warning';
    }
  }

  private formatMoney(microusd: number): string {
    return `$${(microusd / 1_000_000).toFixed(2)}`;
  }

  private toProblem(error: unknown): Problem {
    if (typeof error === 'object' && error !== null && 'code' in error) {
      return error as Problem;
    }
    return { code: 'UNKNOWN', title: 'Unknown error' };
  }
}
