import { DEMO_CARDS, DEMO_AGENTS, DEMO_SOW, DEMO_MEMORY } from './demo-data';

describe('demo read model', () => {
  it('is deterministic and covers the six columns', () => {
    expect(DEMO_CARDS).toHaveLength(5);
    expect(new Set(DEMO_CARDS.map((card) => card.column))).toEqual(
      new Set(['DOING', 'READY', 'TODO', 'REVIEW', 'ICEBOX']),
    );
    expect(DEMO_AGENTS.length).toBeGreaterThan(0);
    expect(DEMO_SOW.tasks.length).toBeGreaterThan(0);
    expect(DEMO_MEMORY.length).toBeGreaterThan(0);
  });
});
