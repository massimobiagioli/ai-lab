import { signal } from '@angular/core';
import { withOptimistic } from './store-utils';

describe('withOptimistic', () => {
  it('keeps the optimistic value on success', async () => {
    const value = signal(1);
    await withOptimistic(value, 2, async () => undefined, 1);
    expect(value()).toBe(2);
  });

  it('rolls back on failure', async () => {
    const value = signal(1);
    await expect(
      withOptimistic(
        value,
        2,
        async () => {
          throw new Error('conflict');
        },
        1,
      ),
    ).rejects.toThrow('conflict');
    expect(value()).toBe(1);
  });
});
