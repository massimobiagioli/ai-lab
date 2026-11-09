import type { Problem } from '../../domain/models';

/** Error carrying an RFC 9457 problem payload. */
export class ApiProblem extends Error implements Problem {
  readonly type?: string;
  readonly title?: string;
  readonly status?: number;
  readonly code: string;
  readonly detail?: string;
  readonly violations?: Problem['violations'];
  readonly trace_id?: string;

  constructor(problem: Problem) {
    super(problem.detail ?? problem.title ?? problem.code);
    this.name = 'ApiProblem';
    this.code = problem.code;
    if (problem.type !== undefined) this.type = problem.type;
    if (problem.title !== undefined) this.title = problem.title;
    if (problem.status !== undefined) this.status = problem.status;
    if (problem.detail !== undefined) this.detail = problem.detail;
    if (problem.violations !== undefined) this.violations = problem.violations;
    if (problem.trace_id !== undefined) this.trace_id = problem.trace_id;
  }
}

/** Normalize an unknown failure into an `ApiProblem`. */
export function toProblem(error: unknown): Problem {
  if (error instanceof ApiProblem) {
    return error;
  }
  const message = error instanceof Error ? error.message : 'Unexpected network error';
  return { code: 'NETWORK_ERROR', title: 'Network error', detail: message };
}
