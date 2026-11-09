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

/** Flow metrics used by the dashboard. */
export interface FlowMetrics {
  readonly leadTimeP50Minutes: number;
  readonly leadTimeP95Minutes: number;
  readonly cycleTimeP50Minutes: number;
  readonly wip: number;
  readonly throughputWeekly: number;
  readonly blockedShare: number;
}
