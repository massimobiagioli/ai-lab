import { animationFor, walkingFrame } from './agent-state';

describe('agent office state', () => {
  it('maps domain states to truthful animations', () => {
    expect(animationFor('idle')).toBe('idle');
    expect(animationFor('working')).toBe('type');
    expect(animationFor('blocked')).toBe('blocked');
    expect(animationFor('paused')).toBe('sleep');
    expect(animationFor('offline')).toBe('sleep');
  });

  it('produces deterministic walking frames', () => {
    expect(walkingFrame(0)).toBe(0);
    expect(walkingFrame(250)).toBe(1);
    expect(walkingFrame(1000)).toBe(1);
    expect(walkingFrame(-100)).toBe(0);
  });
});
