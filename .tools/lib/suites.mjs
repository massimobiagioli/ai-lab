import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const ID_RE = /^E2E-\d{3}$/;

export function loadSuites(root) {
  const file = join(root, '.docs', 'e2e', 'suites.json');
  if (!existsSync(file)) throw new Error(`missing suite manifest: ${file}`);
  const parsed = JSON.parse(readFileSync(file, 'utf8'));
  if (!parsed || typeof parsed !== 'object' || !Array.isArray(parsed.suites)) {
    throw new Error('invalid suite manifest: expected a suites array');
  }
  return parsed.suites;
}

function expandRange(from, to) {
  const start = Number(from.slice(4));
  const end = Number(to.slice(4));
  const ids = [];
  for (let value = start; value <= end; value += 1) ids.push(`E2E-${String(value).padStart(3, '0')}`);
  return ids;
}

export function validateSuites(suites, expected = 480) {
  const errors = [];
  const ids = new Set();
  const suiteByScenario = new Map();
  const seenIds = new Set();
  const seenOrders = new Set();

  for (const suite of suites) {
    if (!suite || typeof suite !== 'object') {
      errors.push('suite entry is not an object');
      continue;
    }
    if (typeof suite.id !== 'string' || !/^[a-z][a-z0-9-]*$/.test(suite.id)) {
      errors.push(`invalid suite id: ${String(suite.id)}`);
    }
    if (seenIds.has(suite.id)) errors.push(`duplicate suite id: ${suite.id}`);
    seenIds.add(suite.id);
    if (!Number.isInteger(suite.order) || suite.order < 1) errors.push(`invalid order for ${suite.id}`);
    if (seenOrders.has(suite.order)) errors.push(`duplicate suite order: ${suite.order}`);
    seenOrders.add(suite.order);
    if (!Array.isArray(suite.ranges) || suite.ranges.length === 0) {
      errors.push(`suite ${suite.id} has no ranges`);
      continue;
    }
    for (const range of suite.ranges) {
      if (!Array.isArray(range) || range.length !== 2 || !ID_RE.test(range[0]) || !ID_RE.test(range[1])) {
        errors.push(`suite ${suite.id} has a malformed range`);
        continue;
      }
      const start = Number(range[0].slice(4));
      const end = Number(range[1].slice(4));
      if (start < 1 || end > expected || start > end) {
        errors.push(`suite ${suite.id} has an out-of-range or reversed range ${range[0]}..${range[1]}`);
        continue;
      }
      for (const id of expandRange(range[0], range[1])) {
        if (ids.has(id)) errors.push(`scenario ${id} belongs to more than one suite`);
        ids.add(id);
        suiteByScenario.set(id, suite.id);
      }
    }
  }

  const expectedIds = new Set(Array.from({ length: expected }, (_, index) => `E2E-${String(index + 1).padStart(3, '0')}`));
  for (const id of expectedIds) {
    if (!ids.has(id)) errors.push(`scenario ${id} is not assigned to a suite`);
  }
  for (const id of ids) {
    if (!expectedIds.has(id)) errors.push(`unknown scenario ${id}`);
  }

  return { errors, suiteByScenario, ids };
}

export function selectSuites(suites, options = {}) {
  const ordered = [...suites].sort((a, b) => a.order - b.order);
  const byId = new Map(ordered.map((suite) => [suite.id, suite]));
  const { suite, upTo } = options;
  if (suite && upTo) throw new Error('choose either a single suite or an up-to suite, not both');
  if (suite) {
    const selected = byId.get(suite);
    if (!selected) throw new Error(`unknown suite: ${suite}`);
    return [selected];
  }
  if (upTo) {
    const target = byId.get(upTo);
    if (!target) throw new Error(`unknown suite: ${upTo}`);
    return ordered.filter((entry) => entry.order <= target.order);
  }
  return ordered;
}

export function grepFor(selected, all) {
  if (all || selected.length === 0) return '';
  if (selected.length === 1) return `@suite-${selected[0].id}`;
  return `@suite-(${selected.map((suite) => suite.id).join('|')})`;
}

export function suiteCounts(suites) {
  return suites.map((suite) => ({
    id: suite.id,
    order: suite.order,
    title: suite.title,
    count: suite.ranges.reduce((total, range) => total + (Number(range[1].slice(4)) - Number(range[0].slice(4)) + 1), 0),
    ranges: suite.ranges,
  }));
}
