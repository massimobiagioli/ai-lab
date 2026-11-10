// Unit tests for the BDD suite split.
//
// Run with: node scripts/bdd-suites.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { grepFor, loadSuites, selectSuites, suiteCounts, validateSuites } from '../.tools/lib/suites.mjs';

const HERE = fileURLToPath(new URL('.', import.meta.url));
const ROOT = resolve(HERE, '..');
const SCRIPT = resolve(HERE, 'bdd-suites.mjs');

test('the manifest defines six suites covering 480 scenarios', () => {
  const suites = loadSuites(ROOT);
  const validation = validateSuites(suites);
  assert.deepEqual(validation.errors, []);
  const counts = suiteCounts(suites);
  assert.equal(counts.length, 6);
  assert.equal(counts.reduce((sum, suite) => sum + suite.count, 0), 480);
  assert.deepEqual(counts.map((suite) => suite.id), ['core', 'kanban', 'delivery', 'platform', 'office', 'management']);
});

test('a single suite selects only that suite', () => {
  const suites = loadSuites(ROOT);
  const selected = selectSuites(suites, { suite: 'core' });
  assert.deepEqual(selected.map((suite) => suite.id), ['core']);
  assert.equal(grepFor(selected, false), '@suite-core');
});

test('up-to selects every suite up to the target', () => {
  const suites = loadSuites(ROOT);
  const selected = selectSuites(suites, { upTo: 'delivery' });
  assert.deepEqual(selected.map((suite) => suite.id), ['core', 'kanban', 'delivery']);
  assert.equal(grepFor(selected, false), '@suite-(core|kanban|delivery)');
});

test('no selection means all suites', () => {
  const suites = loadSuites(ROOT);
  const selected = selectSuites(suites, {});
  assert.equal(selected.length, 6);
  assert.equal(grepFor(selected, true), '');
});

test('an unknown suite is rejected', () => {
  const suites = loadSuites(ROOT);
  assert.throws(() => selectSuites(suites, { suite: 'unknown' }), /unknown suite/);
  assert.throws(() => selectSuites(suites, { upTo: 'unknown' }), /unknown suite/);
});

test('a single suite and an up-to selection cannot be combined', () => {
  const suites = loadSuites(ROOT);
  assert.throws(() => selectSuites(suites, { suite: 'core', upTo: 'delivery' }), /not both/);
});

test('overlapping or missing ids are rejected', () => {
  const suites = [
    { id: 'one', order: 1, ranges: [['E2E-001', 'E2E-003']] },
    { id: 'two', order: 2, ranges: [['E2E-003', 'E2E-004']] },
  ];
  const validation = validateSuites(suites, 4);
  assert.ok(validation.errors.some((error) => /more than one suite/.test(error)));
  assert.ok(validation.errors.some((error) => /E2E-002/.test(error)) === false);
});

test('the CLI lists the suites and resolves a selection', () => {
  const list = spawnSync(process.execPath, [SCRIPT, '--list'], { encoding: 'utf8' });
  assert.equal(list.status, 0);
  assert.match(list.stdout, /core/);
  assert.match(list.stdout, /management/);
  assert.match(list.stdout, /Total: 480 scenarios/);

  const selection = spawnSync(process.execPath, [SCRIPT, '--suite', 'core', '--json'], { encoding: 'utf8' });
  assert.equal(selection.status, 0);
  const payload = JSON.parse(selection.stdout);
  assert.deepEqual(payload.suites, ['core']);
  assert.equal(payload.total, 41);
  assert.equal(payload.grep, '@suite-core');
});

test('the CLI rejects an unknown suite', () => {
  const result = spawnSync(process.execPath, [SCRIPT, '--suite', 'unknown'], { encoding: 'utf8' });
  assert.equal(result.status, 2);
  assert.match(result.stderr, /unknown suite/);
});
