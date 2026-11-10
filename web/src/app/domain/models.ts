/** Cursor page envelope shared by the edge contract. */
export interface Page<T> {
  readonly data: readonly T[];
  readonly next_cursor: string | null;
  readonly total_estimate: number;
}

/** Run lifecycle state. */
export type RunState = 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled' | 'timed_out';

/** A run view from the edge API. */
export interface RunView {
  readonly id: string;
  readonly card_id: string;
  readonly lab_id: string;
  readonly state: RunState;
  readonly attempt: number;
  readonly started_at: string | null;
  readonly ended_at: string | null;
  readonly cost_microusd: number;
  readonly created_at: string;
}

/** A run step view. */
export interface StepView {
  readonly seq: number;
  readonly stage: string;
  readonly kind: string;
  readonly exit_code: number | null;
  readonly duration_ms: number | null;
  readonly output: string;
}

/** RFC 9457 problem payload. */
export interface Problem {
  readonly type?: string | undefined;
  readonly title?: string | undefined;
  readonly status?: number | undefined;
  readonly code: string;
  readonly detail?: string | undefined;
  readonly violations?:
    | ReadonlyArray<{
        readonly rule: string;
        readonly field: string;
        readonly message: string;
      }>
    | undefined;
  readonly trace_id?: string | undefined;
}

/** A WebSocket event envelope. */
export interface EventEnvelope {
  readonly event_id: string;
  readonly event: string;
  readonly seq: number;
  readonly lab_id: string;
  readonly occurred_at: string;
  readonly payload: Record<string, unknown>;
}

/** A lab summary used by the dashboard selector. */
export interface LabSummary {
  readonly id: string;
  readonly name: string;
  readonly slug: string;
  readonly autonomyLevel: number;
  readonly monthlyBudgetMicrousd: number;
}

/** Acknowledgement returned by the idea composer. */
export interface IdeaAck {
  readonly id: string;
  readonly status: string;
}

/** Flow metrics used by the dashboard. */
export interface FlowMetrics {
  readonly leadTimeP50Minutes: number;
  readonly leadTimeP95Minutes: number;
  readonly cycleTimeP50Minutes: number;
  readonly wip: number;
  readonly throughputWeekly: number;
  readonly blockedShare: number;
}

/** A Kanban card. */
export interface Card {
  readonly id: string;
  readonly key: string;
  readonly projectId: string;
  readonly title: string;
  readonly column: string;
  readonly priority: 'must' | 'should' | 'could' | 'wont';
  readonly assigneeId: string | null;
  readonly labels: readonly string[];
  readonly estimate: number | null;
  readonly blockedReason: string | null;
  readonly prUrl: string | null;
}

/** An agent profile. */
export interface Agent {
  readonly id: string;
  readonly name: string;
  readonly role: 'tech_lead' | 'analyst' | 'dev' | 'qa';
  readonly status: 'idle' | 'working' | 'blocked' | 'paused' | 'offline';
  readonly autonomyLevel: number | null;
  readonly model: string;
  readonly maxConcurrent: number;
}

/** A project bound to a lab and repository. */
export interface Project {
  readonly id: string;
  readonly labId: string;
  readonly name: string;
  readonly repo: string;
  readonly branch: string;
  readonly autonomyLevel: number;
}

/** A provider credential view (secrets are write-only). */
export interface Provider {
  readonly id: string;
  readonly name: string;
  readonly kind: 'deepseek';
  readonly status: 'active' | 'invalid' | 'revoked';
  readonly last4: string;
}

/** A SOW section. */
export interface SowSection {
  readonly id: string;
  readonly title: string;
  readonly body: string;
}

/** A SOW task. */
export interface SowTask {
  readonly id: string;
  readonly title: string;
  readonly status: 'todo' | 'doing' | 'done';
  readonly dependsOn: readonly string[];
}

/** A SOW document. */
export interface Sow {
  readonly id: string;
  readonly projectId: string;
  readonly version: number;
  readonly status: 'draft' | 'approved' | 'rejected';
  readonly sections: readonly SowSection[];
  readonly tasks: readonly SowTask[];
}

/** A memory note. */
export interface MemoryNote {
  readonly id: string;
  readonly scope: 'lab' | 'project' | 'agent';
  readonly title: string;
  readonly snippet: string;
  readonly citations: readonly string[];
}

/** Lab settings. */
export interface LabSettings {
  readonly labId: string;
  readonly autonomyLevel: number;
  readonly monthlyBudgetMicrousd: number;
  readonly schedulerTickMs: number;
  readonly officeTheme: 'default' | 'high-contrast';
}
