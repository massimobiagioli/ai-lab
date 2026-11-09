export type AgentDomainState = 'idle' | 'working' | 'blocked' | 'paused' | 'offline';
export type OfficeAnimation = 'idle' | 'type' | 'blocked' | 'sleep';

/** Map the domain state to a truthful office animation. */
export function animationFor(state: AgentDomainState): OfficeAnimation {
  switch (state) {
    case 'idle':
      return 'idle';
    case 'working':
      return 'type';
    case 'blocked':
      return 'blocked';
    case 'paused':
    case 'offline':
      return 'sleep';
  }
}

/** Deterministic walking frame index. */
export function walkingFrame(elapsedMs: number): number {
  return Math.floor(Math.max(0, elapsedMs) / 200) % 4;
}
