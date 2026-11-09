import type { WritableSignal } from '@angular/core';

/** Explicit async state for every feature store. */
export type LoadState = 'idle' | 'loading' | 'ready' | 'error';

/**
 * Apply an optimistic value, run the action and roll back on failure.
 * The caller owns the error handling after the rollback.
 */
export async function withOptimistic<T>(
  target: WritableSignal<T>,
  optimistic: T,
  action: () => Promise<void>,
  rollback: T,
): Promise<void> {
  target.set(optimistic);
  try {
    await action();
  } catch (error) {
    target.set(rollback);
    throw error;
  }
}
