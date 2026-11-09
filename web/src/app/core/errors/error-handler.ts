import { ErrorHandler, inject, Injectable } from '@angular/core';
import { Telemetry } from '../telemetry/telemetry.service';
import { ErrorState } from './error-state';

/** Global error boundary that records telemetry and exposes a retryable state. */
@Injectable()
export class AppErrorHandler implements ErrorHandler {
  private readonly state = inject(ErrorState);
  private readonly telemetry = inject(Telemetry);

  handleError(error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);
    const traceId = crypto.randomUUID();
    this.state.set(message.slice(0, 240), traceId);
    this.telemetry.capture('error', message, error);
  }
}
