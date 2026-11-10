import { Component, computed, effect, inject, signal } from '@angular/core';
import { Router, RouterLink } from '@angular/router';
import { ApiClient } from '../../core/api/api-client';
import { sessionToken } from '../../core/auth/session';
import { RUNTIME_CONFIG } from '../../core/config/runtime-config';
import { PageHeader } from '../../shared/ui/page-header/page-header';
import { Card } from '../../shared/ui/card/card';
import { Button } from '../../shared/ui/button/button';
import { Input } from '../../shared/ui/input/input';
import { Select } from '../../shared/ui/select/select';
import { Progress } from '../../shared/ui/progress/progress';
import { CHARACTERS } from '../../../lib/office/characters';
import type { Problem } from '../../domain/models';

const STEPS = [
  { id: 'provider', label: 'Provider', hint: 'DeepSeek key and endpoint' },
  { id: 'lab', label: 'Lab and Tech Lead', hint: 'The team that runs the work' },
  { id: 'project', label: 'Project', hint: 'Repository and card prefix' },
  { id: 'idea', label: 'First idea', hint: 'Turn intent into cards' },
] as const;

const TECH_LEAD_DESCRIPTION =
  'Owns delivery order, architecture, decomposition and merge decisions. Refines ideas into canonical cards, enforces DoR and DoD, and escalates budget or scope risks before they become blockers.';

@Component({
  selector: 'app-onboarding',
  imports: [PageHeader, Card, Button, Input, Select, Progress, RouterLink],
  template: `
    <app-page-header
      title="Set up AI-Lab"
      subtitle="Load the minimum data to start delivering. You can change everything later."
    >
      <a class="skip-link" routerLink="/">Skip for now</a>
    </app-page-header>

    <app-card>
      <div class="token">
        <app-input label="API token" type="password" [(value)]="apiToken" />
        <p class="token__hint">
          From <code>AILAB_API_TOKEN</code> in <code>.env</code>. It is stored only in this tab,
          never written to disk.
        </p>
      </div>
    </app-card>

    <app-card>
      <div class="progress">
        <div>
          <p class="progress__count">Step {{ stepIndex() + 1 }} of {{ steps.length }}</p>
          <p class="progress__hint">{{ currentStep().hint }}</p>
        </div>
        <app-progress [value]="progress()" label="Setup progress" />
      </div>
    </app-card>

    <div class="setup">
      <aside class="setup__steps" aria-label="Setup steps">
        @for (item of steps; track item.id; let index = $index) {
          <div
            class="setup__step"
            [class.is-active]="index === stepIndex()"
            [class.is-done]="index < stepIndex()"
          >
            <span class="setup__index" aria-hidden="true">{{
              index < stepIndex() ? '✓' : index + 1
            }}</span>
            <div>
              <p>{{ item.label }}</p>
              <small>{{ item.hint }}</small>
            </div>
          </div>
        }
      </aside>

      <section class="setup__panel">
        @if (done()) {
          <app-card title="You are ready to deliver">
            <p class="muted">
              The provider, lab, Tech Lead and project are in place. Open the dashboard and let the
              team start from the first idea.
            </p>
            <div class="actions">
              <app-button variant="primary" (pressed)="finish()">Open dashboard</app-button>
            </div>
          </app-card>
        } @else {
          @switch (currentStep().id) {
            @case ('provider') {
              <app-card title="Connect DeepSeek">
                <p class="muted">
                  The key is encrypted and never shown again. Use a funded key with access to the
                  DeepSeek API.
                </p>
                <div class="fields">
                  <app-input label="Provider name" [(value)]="providerName" />
                  <app-input label="Base URL" [(value)]="providerBaseUrl" />
                  <app-input label="API key" type="password" [(value)]="providerKey" />
                </div>
                <div class="actions">
                  <app-button
                    variant="primary"
                    [loading]="busy()"
                    [disabled]="!providerValid()"
                    (pressed)="saveProvider()"
                  >
                    Save provider
                  </app-button>
                </div>
              </app-card>
            }
            @case ('lab') {
              <app-card title="Create the lab and Tech Lead">
                <p class="muted">
                  Every lab needs exactly one active Tech Lead. The character is a visual persona,
                  not a permission.
                </p>
                <div class="fields">
                  <app-input label="Lab name" [(value)]="labName" />
                  <app-input label="Slug" [(value)]="labSlug" placeholder="my-lab" />
                  <app-select label="Autonomy" [options]="autonomyOptions" [(value)]="autonomy" />
                  <app-input label="Monthly budget (micro-USD)" [(value)]="budget" />
                </div>
                <h4>Tech Lead</h4>
                <div class="fields">
                  <app-input label="Tech Lead name" [(value)]="techLeadName" />
                  <app-select
                    label="Character"
                    [options]="characterOptions"
                    [(value)]="character"
                  />
                </div>
                <p class="character-note">{{ characterSpeech() }}</p>
                <div class="actions">
                  <app-button (pressed)="back()">Back</app-button>
                  <app-button
                    variant="primary"
                    [loading]="busy()"
                    [disabled]="!labValid()"
                    (pressed)="saveLab()"
                  >
                    Create lab
                  </app-button>
                </div>
              </app-card>
            }
            @case ('project') {
              <app-card title="Connect the first project">
                <p class="muted">
                  The project binds a GitHub repository to the lab and creates its board.
                </p>
                <div class="fields">
                  <app-input label="Project name" [(value)]="projectName" />
                  <app-input label="Slug" [(value)]="projectSlug" placeholder="my-project" />
                  <app-input label="Key prefix" [(value)]="keyPrefix" placeholder="AIL" />
                  <app-input label="Repository" [(value)]="sourceRef" placeholder="owner/repo" />
                  <app-input label="Default branch" [(value)]="defaultBranch" />
                </div>
                <div class="actions">
                  <app-button (pressed)="back()">Back</app-button>
                  <app-button
                    variant="primary"
                    [loading]="busy()"
                    [disabled]="!projectValid()"
                    (pressed)="saveProject()"
                  >
                    Create project
                  </app-button>
                </div>
              </app-card>
            }
            @default {
              <app-card title="Send the first idea">
                <p class="muted">
                  Describe the outcome. The Tech Lead turns it into estimable cards on the board.
                </p>
                <textarea
                  class="idea"
                  rows="5"
                  [value]="idea()"
                  (input)="onIdeaInput($event)"
                  placeholder="Example: Build a weekly customer feedback digest with a review queue."
                ></textarea>
                <div class="actions">
                  <app-button (pressed)="back()">Back</app-button>
                  <app-button
                    variant="primary"
                    [loading]="busy()"
                    [disabled]="idea().trim().length < 10"
                    (pressed)="sendIdea()"
                  >
                    Send to Tech Lead
                  </app-button>
                </div>
              </app-card>
            }
          }
        }

        @if (message() !== '') {
          <div class="notice notice--success" role="status">{{ message() }}</div>
        }
        @if (error() !== null) {
          <div class="notice notice--error" role="alert">
            <strong>{{ error()?.code }}</strong>
            <span>{{
              error()?.detail ?? error()?.title ?? 'The step could not be completed.'
            }}</span>
          </div>
        }
      </section>
    </div>
  `,
  styles: `
    .skip-link {
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
      font-weight: 600;
      text-decoration: none;
    }
    .token {
      display: grid;
      grid-template-columns: minmax(240px, 0.8fr) minmax(0, 1fr);
      gap: var(--ailab-space-3);
      align-items: end;
    }
    .token__hint {
      margin: 0 0 0.4rem;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .token__hint code {
      padding: 0.1rem 0.25rem;
      border-radius: 4px;
      background: var(--ailab-color-surface-sunken);
      font-size: 0.78em;
    }
    .progress {
      display: flex;
      align-items: center;
      gap: var(--ailab-space-4);
    }
    .progress app-progress {
      flex: 1;
    }
    .progress__count {
      margin: 0;
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-sm);
      font-weight: 700;
    }
    .progress__hint {
      margin: 0.1rem 0 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-xs);
    }
    .setup {
      display: grid;
      grid-template-columns: minmax(220px, 0.7fr) minmax(0, 1.6fr);
      gap: var(--ailab-space-6);
      margin-top: var(--ailab-space-6);
      align-items: start;
    }
    .setup__steps {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-2);
    }
    .setup__step {
      display: flex;
      gap: var(--ailab-space-3);
      padding: var(--ailab-space-3);
      border: 1px solid transparent;
      border-radius: var(--ailab-radius-md);
      color: var(--ailab-color-muted);
    }
    .setup__step.is-active {
      border-color: var(--ailab-color-accent-soft);
      background: var(--ailab-color-accent-soft);
      color: var(--ailab-color-accent);
    }
    .setup__step.is-done {
      color: var(--ailab-color-success);
    }
    .setup__index {
      display: grid;
      place-items: center;
      width: 1.6rem;
      height: 1.6rem;
      border: 1px solid currentColor;
      border-radius: 50%;
      font-size: var(--ailab-font-size-xs);
      font-weight: 700;
      flex: none;
    }
    .setup__step p {
      margin: 0;
      font-size: var(--ailab-font-size-sm);
      font-weight: 650;
    }
    .setup__step small {
      display: block;
      margin-top: 0.1rem;
      font-size: var(--ailab-font-size-xs);
      opacity: 0.8;
    }
    .setup__panel {
      display: flex;
      flex-direction: column;
      gap: var(--ailab-space-4);
      min-width: 0;
    }
    .fields {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: var(--ailab-space-3);
      margin-top: var(--ailab-space-3);
    }
    .fields app-input:nth-child(3) {
      grid-column: 1 / -1;
    }
    .actions {
      display: flex;
      justify-content: flex-end;
      gap: var(--ailab-space-2);
      margin-top: var(--ailab-space-4);
    }
    h4 {
      margin: var(--ailab-space-4) 0 0;
      color: var(--ailab-color-text);
      font-size: var(--ailab-font-size-sm);
      font-weight: 700;
    }
    .character-note {
      margin: var(--ailab-space-2) 0 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    .idea {
      width: 100%;
      margin-top: var(--ailab-space-3);
      padding: 0.8rem 0.9rem;
      border: 1px solid var(--ailab-color-border);
      border-radius: var(--ailab-radius-md);
      background: var(--ailab-color-surface);
      color: var(--ailab-color-text);
      font: inherit;
      resize: vertical;
    }
    .notice {
      display: flex;
      flex-direction: column;
      gap: 0.2rem;
      padding: var(--ailab-space-3);
      border-radius: var(--ailab-radius-md);
      font-size: var(--ailab-font-size-sm);
    }
    .notice--success {
      border: 1px solid var(--ailab-color-success-soft);
      background: var(--ailab-color-success-soft);
      color: var(--ailab-color-success);
    }
    .notice--error {
      border: 1px solid var(--ailab-color-danger-soft);
      background: var(--ailab-color-danger-soft);
      color: var(--ailab-color-danger);
    }
    .muted {
      margin-top: 0;
      color: var(--ailab-color-muted);
      font-size: var(--ailab-font-size-sm);
    }
    @media (max-width: 760px) {
      .token {
        grid-template-columns: 1fr;
      }
      .setup {
        grid-template-columns: 1fr;
      }
      .fields {
        grid-template-columns: 1fr;
      }
      .fields app-input:nth-child(3) {
        grid-column: auto;
      }
    }
  `,
})
export class Onboarding {
  private readonly api = inject(ApiClient);
  private readonly router = inject(Router);
  private readonly config = inject(RUNTIME_CONFIG);

  protected readonly steps = STEPS;
  protected readonly apiToken = signal(sessionToken.read() ?? this.config.token ?? '');
  protected readonly stepIndex = signal(0);
  protected readonly currentStep = computed(() => STEPS[this.stepIndex()] ?? STEPS[0]!);
  protected readonly done = computed(() => this.stepIndex() >= STEPS.length);
  protected readonly progress = computed(() =>
    Math.min(100, ((this.stepIndex() + 1) / STEPS.length) * 100),
  );
  protected readonly busy = signal(false);
  protected readonly error = signal<Problem | null>(null);
  protected readonly message = signal('');

  protected readonly providerName = signal('DeepSeek');
  protected readonly providerBaseUrl = signal('https://api.deepseek.com');
  protected readonly providerKey = signal('');
  protected readonly labName = signal('AI-Lab');
  protected readonly labSlug = signal('ai-lab');
  protected readonly autonomy = signal('2');
  protected readonly budget = signal('50000000');
  protected readonly techLeadName = signal('Tech Lead');
  protected readonly character = signal('veteran_lead');
  protected readonly projectName = signal('AI-Lab Project');
  protected readonly projectSlug = signal('ai-lab-project');
  protected readonly keyPrefix = signal('AIL');
  protected readonly sourceRef = signal('owner/repository');
  protected readonly defaultBranch = signal('main');
  protected readonly idea = signal('');
  protected labId: string | null = null;

  protected readonly autonomyOptions = [
    { value: '0', label: 'A0 — manual' },
    { value: '1', label: 'A1 — assisted' },
    { value: '2', label: 'A2 — supervised' },
    { value: '3', label: 'A3 — autonomous' },
  ] as const;
  protected readonly characterOptions = CHARACTERS.map((profile) => ({
    value: profile.id,
    label: profile.name,
  }));
  protected readonly characterSpeech = computed(() => {
    const profile = CHARACTERS.find((item) => item.id === this.character());
    return profile?.speech ?? '';
  });

  constructor() {
    effect(() => {
      sessionToken.write(this.apiToken().trim() === '' ? null : this.apiToken().trim());
    });
  }

  protected providerValid(): boolean {
    return (
      this.providerName().trim().length > 0 &&
      this.providerBaseUrl().startsWith('https://') &&
      this.providerKey().startsWith('sk-') &&
      this.providerKey().length >= 20
    );
  }

  protected labValid(): boolean {
    return (
      this.labName().trim().length >= 2 &&
      this.labSlug().trim().length >= 2 &&
      this.techLeadName().trim().length >= 2
    );
  }

  protected projectValid(): boolean {
    return (
      this.projectName().trim().length >= 2 &&
      this.projectSlug().trim().length >= 2 &&
      /^[A-Z][A-Z0-9]{1,9}$/.test(this.keyPrefix()) &&
      this.sourceRef().includes('/') &&
      this.labId !== null
    );
  }

  protected async saveProvider(): Promise<void> {
    await this.run(async () => {
      await this.api.createProvider({
        name: this.providerName().trim(),
        kind: 'deepseek',
        base_url: this.providerBaseUrl().trim(),
        api_key: this.providerKey(),
      });
      this.message.set('Provider saved. The key is encrypted.');
      this.stepIndex.set(1);
    });
  }

  protected async saveLab(): Promise<void> {
    await this.run(async () => {
      const response = await this.api.createLab({
        name: this.labName().trim(),
        slug: this.labSlug().trim(),
        description: 'Lab created from the setup assistant.',
        autonomy_level: Number(this.autonomy()),
        monthly_budget_microusd: Number(this.budget()),
        tech_lead: {
          name: this.techLeadName().trim(),
          role: 'tech_lead',
          character_id: this.character(),
          description: TECH_LEAD_DESCRIPTION,
        },
      });
      this.labId = response.lab.id;
      this.message.set('Lab and Tech Lead created.');
      this.stepIndex.set(2);
    });
  }

  protected async saveProject(): Promise<void> {
    if (this.labId === null) {
      this.error.set({ code: 'LAB_REQUIRED', title: 'Create the lab first.' });
      return;
    }
    await this.run(async () => {
      await this.api.createProject({
        lab_id: this.labId!,
        name: this.projectName().trim(),
        slug: this.projectSlug().trim(),
        key_prefix: this.keyPrefix().trim().toUpperCase(),
        description: 'Project created from the setup assistant.',
        source_kind: 'github',
        source_ref: this.sourceRef().trim(),
        default_branch: this.defaultBranch().trim(),
      });
      this.message.set('Project and board created.');
      this.stepIndex.set(3);
    });
  }

  protected async sendIdea(): Promise<void> {
    await this.run(async () => {
      await this.api.submitIdea(this.idea().trim());
      this.message.set('Idea queued. The Tech Lead will shape it into cards.');
      this.stepIndex.set(STEPS.length);
    });
  }

  protected back(): void {
    this.error.set(null);
    this.message.set('');
    this.stepIndex.update((value) => Math.max(0, value - 1));
  }

  protected onIdeaInput(event: Event): void {
    const target = event.target;
    if (target instanceof HTMLTextAreaElement) {
      this.idea.set(target.value);
    }
  }

  protected finish(): void {
    void this.router.navigate(['/']);
  }

  private async run(work: () => Promise<void>): Promise<void> {
    this.busy.set(true);
    this.error.set(null);
    this.message.set('');
    try {
      await work();
    } catch (error) {
      this.error.set(this.toProblem(error));
    } finally {
      this.busy.set(false);
    }
  }

  private toProblem(error: unknown): Problem {
    if (typeof error === 'object' && error !== null && 'code' in error) {
      return error as Problem;
    }
    return { code: 'UNKNOWN', title: 'The step could not be completed.' };
  }
}
