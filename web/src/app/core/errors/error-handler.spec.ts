import { TestBed } from '@angular/core/testing';
import { Telemetry } from '../telemetry/telemetry.service';
import { AppErrorHandler } from './error-handler';
import { ErrorState } from './error-state';

describe('AppErrorHandler', () => {
  it('records the error and a trace id', () => {
    const telemetry = { capture: () => undefined };
    TestBed.configureTestingModule({
      providers: [ErrorState, AppErrorHandler, { provide: Telemetry, useValue: telemetry }],
    });
    const state = TestBed.inject(ErrorState);
    const handler = TestBed.inject(AppErrorHandler);
    handler.handleError(new Error('boom'));
    expect(state.error()?.message).toBe('boom');
    expect(state.error()?.traceId).toBeTruthy();
  });
});
