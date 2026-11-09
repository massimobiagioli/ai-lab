import type {
  Agent,
  Card,
  LabSettings,
  MemoryNote,
  Project,
  Provider,
  Sow,
} from '../../domain/models';
import { ApiProblem } from '../api/problem';

/** Demo read model used only while the edge endpoints are 501 stubs. */
export const AILAB_DEMO_DATA = true;

/** The edge answers 404/501 until WP-13B T07; treat those as the demo signal. */
export function isDemoFallback(error: unknown): boolean {
  return (
    error instanceof ApiProblem &&
    (error.status === 404 || error.status === 501 || error.code === 'NOT_IMPLEMENTED')
  );
}

export const DEMO_PROJECT: Project = {
  id: 'demo',
  labId: 'default',
  name: 'AI-Lab Demo',
  repo: 'ai-lab/demo',
  branch: 'main',
  autonomyLevel: 2,
};

export const DEMO_CARDS: readonly Card[] = [
  {
    id: 'card-1',
    key: 'AIL-1',
    projectId: 'demo',
    title: 'Build the Kanban board',
    column: 'DOING',
    priority: 'must',
    assigneeId: 'agent-dev',
    labels: ['ui'],
    estimate: 3,
    blockedReason: null,
    prUrl: 'https://github.com/ai-lab/demo/pull/1',
  },
  {
    id: 'card-2',
    key: 'AIL-2',
    projectId: 'demo',
    title: 'Inventory wizards',
    column: 'READY',
    priority: 'should',
    assigneeId: 'agent-dev',
    labels: ['ui'],
    estimate: 5,
    blockedReason: null,
    prUrl: null,
  },
  {
    id: 'card-3',
    key: 'AIL-3',
    projectId: 'demo',
    title: 'Memory explorer',
    column: 'TODO',
    priority: 'should',
    assigneeId: null,
    labels: ['memory'],
    estimate: 3,
    blockedReason: null,
    prUrl: null,
  },
  {
    id: 'card-4',
    key: 'AIL-4',
    projectId: 'demo',
    title: 'Release 1.0',
    column: 'REVIEW',
    priority: 'must',
    assigneeId: 'agent-qa',
    labels: ['release'],
    estimate: 8,
    blockedReason: null,
    prUrl: 'https://github.com/ai-lab/demo/pull/4',
  },
  {
    id: 'card-5',
    key: 'AIL-5',
    projectId: 'demo',
    title: 'Retired provider',
    column: 'ICEBOX',
    priority: 'could',
    assigneeId: null,
    labels: ['risk'],
    estimate: 1,
    blockedReason: 'risk: provider retired',
    prUrl: null,
  },
];

export const DEMO_AGENTS: readonly Agent[] = [
  {
    id: 'agent-tech-lead',
    name: 'Tech Lead',
    role: 'tech_lead',
    status: 'idle',
    autonomyLevel: 2,
    model: 'deepseek-v4',
    maxConcurrent: 1,
  },
  {
    id: 'agent-analyst',
    name: 'Analyst',
    role: 'analyst',
    status: 'idle',
    autonomyLevel: 2,
    model: 'deepseek-v4-pro',
    maxConcurrent: 1,
  },
  {
    id: 'agent-dev',
    name: 'Dev',
    role: 'dev',
    status: 'working',
    autonomyLevel: 2,
    model: 'deepseek-flash',
    maxConcurrent: 2,
  },
  {
    id: 'agent-qa',
    name: 'QA',
    role: 'qa',
    status: 'working',
    autonomyLevel: 1,
    model: 'deepseek-v4-pro',
    maxConcurrent: 1,
  },
];

export const DEMO_PROJECTS: readonly Project[] = [DEMO_PROJECT];

export const DEMO_PROVIDERS: readonly Provider[] = [
  { id: 'provider-1', name: 'DeepSeek', kind: 'deepseek', status: 'active', last4: 'd3f1' },
];

export const DEMO_SOW: Sow = {
  id: 'sow-demo',
  projectId: 'demo',
  version: 3,
  status: 'approved',
  sections: [
    {
      id: 'goal',
      title: 'Goal',
      body: 'Deliver the AI-Lab product UI surfacing the lab state truthfully.',
    },
    { id: 'scope', title: 'Scope', body: 'Dashboard, Kanban, inventory, SOW, memory, settings.' },
  ],
  tasks: [
    { id: 't1', title: 'Data layer', status: 'done', dependsOn: [] },
    { id: 't2', title: 'Kanban', status: 'doing', dependsOn: ['t1'] },
    { id: 't3', title: 'Inventory', status: 'todo', dependsOn: ['t1'] },
    { id: 't4', title: 'Release', status: 'todo', dependsOn: ['t2', 't3'] },
  ],
};

export const DEMO_MEMORY: readonly MemoryNote[] = [
  {
    id: 'mem-1',
    scope: 'project',
    title: 'Board conventions',
    snippet: 'Columns follow the six-state matrix.',
    citations: ['docs://kanban'],
  },
  {
    id: 'mem-2',
    scope: 'agent',
    title: 'QA heuristics',
    snippet: 'Verify acceptance criteria with independent evidence.',
    citations: ['docs://qa-merge'],
  },
];

export const DEMO_SETTINGS: LabSettings = {
  labId: 'default',
  autonomyLevel: 2,
  monthlyBudgetMicrousd: 50_000_000,
  schedulerTickMs: 1_000,
  officeTheme: 'default',
};
