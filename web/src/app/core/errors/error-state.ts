import { Injectable, signal } from '@angular/core';

/** A fatal UI error surfaced by the shell error boundary. */
export interface UiError {
  readonly message: string;
  readonly traceId: string;
}

/** Holds the current fatal error for the shell boundary. */
@Injectable({ providedIn: 'root' })
export class ErrorState {
  readonly error = signal<UiError | null>(null);

  set(message: string, traceId: string): void {
    this.error.set({ message, traceId });
  }

  clear(): void {
    this.error.set(null);
  }
}
