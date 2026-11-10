// Unit tests for scripts/bdd-preflight.mjs.
//
// Run with: node scripts/bdd-preflight.test.mjs
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { evaluateChecklist, formatHuman, parseChecklist } from './bdd-preflight.mjs';

const HERE = fileURLToPath(new URL('.', import.meta.url));
const SCRIPT = resolve(HERE, 'bdd-preflight.mjs');

function runCli(args, stdin) {
  return spawnSync(process.execPath, [SCRIPT, ...args], { encoding: 'utf8', input: stdin });
}

test('all checked items are satisfied', () => {
  const result = evaluateChecklist('- [x] api-key\n- [x] other\n');
  assert.equal(result.ok, true);
  assert.equal(result.total, 2);
  assert.equal(result.missing.length, 0);
});

test('one unchecked item is missing', () => {
  const result = evaluateChecklist('- [x] api-key\n- [ ] other\n');
  assert.equal(result.ok, false);
  assert.deepEqual(result.missing, [{ line: 2, text: 'other', marker: ' ' }]);
});

test('N/A satisfies an unchecked item', () => {
  const result = evaluateChecklist('- [ ] optional step N/A\n');
  assert.equal(result.ok, true);
  assert.equal(result.checked, 1);
});

test('fenced checkboxes are ignored', () => {
  const markdown = ['- [x] ready', '```text', '- [ ] ignored', '```', '- [x] ready too'].join('\n');
  const items = parseChecklist(markdown);
  assert.equal(items.length, 2);
  assert.equal(items.every((item) => item.checked), true);
});

test('extra checked items do not block', () => {
  const result = evaluateChecklist('- [x] api-key\n- [x] extra note\n');
  assert.equal(result.ok, true);
});

test('exclamation and dash markers are not satisfied', () => {
  const result = evaluateChecklist('- [!] failed\n- [-] blocked\n');
  assert.equal(result.ok, false);
  assert.equal(result.missing.length, 2);
});

test('human message lists every missing item with its line', () => {
  const result = evaluateChecklist('- [x] ready\n- [ ] api-key\n- [ ] other\n');
  const text = formatHuman(result, '.docs/e2e/req.md');
  assert.match(text, /BDD suite not started/);
  assert.match(text, /line 2: api-key/);
  assert.match(text, /line 3: other/);
});

test('CLI exits 2 for an unreadable checklist', () => {
  const result = runCli(['--file', '/definitely/missing/req.md']);
  assert.equal(result.status, 2);
  assert.match(result.stderr, /missing or unreadable/);
});

test('CLI exits 0 for a ready checklist and emits JSON', () => {
  const dir = mkdtempSync(join(tmpdir(), 'ailab-bdd-preflight-'));
  try {
    const file = join(dir, 'req.md');
    writeFileSync(file, '- [x] api-key\n', 'utf8');
    const result = runCli(['--file', file, '--json']);
    assert.equal(result.status, 0);
    const payload = JSON.parse(result.stdout);
    assert.equal(payload.ok, true);
    assert.equal(payload.checked, 1);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('CLI exits 2 for an unchecked checklist', () => {
  const dir = mkdtempSync(join(tmpdir(), 'ailab-bdd-preflight-'));
  try {
    const file = join(dir, 'req.md');
    writeFileSync(file, '- [ ] api-key\n', 'utf8');
    const result = runCli(['--file', file]);
    assert.equal(result.status, 2);
    assert.match(result.stderr, /Missing items/);
    assert.match(result.stderr, /line 1: api-key/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});


test('live-only items are satisfied for fake runs', () => {
  const markdown = '- [ ] (live) DeepSeek API key\n';
  const fake = evaluateChecklist(markdown);
  assert.equal(fake.ok, true);
  assert.equal(fake.checked, 1);
  assert.equal(fake.live, false);
});

test('live-only items block live runs when unchecked', () => {
  const markdown = '- [ ] (live) DeepSeek API key\n';
  const live = evaluateChecklist(markdown, { live: true });
  assert.equal(live.ok, false);
  assert.equal(live.live, true);
  assert.deepEqual(live.missing, [
    { line: 1, text: '(live) DeepSeek API key', marker: ' ' },
  ]);
});

test('live mode still blocks non-live unchecked items', () => {
  const result = evaluateChecklist('- [x] (live) key\n- [ ] database\n', { live: true });
  assert.equal(result.ok, false);
  assert.equal(result.missing.length, 1);
  assert.match(result.missing[0].text, /database/);
});

test('CLI allows an unchecked live item without --live', () => {
  const dir = mkdtempSync(join(tmpdir(), 'ailab-bdd-preflight-'));
  try {
    const file = join(dir, 'req.md');
    writeFileSync(file, '- [ ] (live) DeepSeek API key\n', 'utf8');
    const result = runCli(['--file', file]);
    assert.equal(result.status, 0);
    assert.match(result.stdout, /BDD preflight: ready/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('CLI blocks an unchecked live item with --live', () => {
  const dir = mkdtempSync(join(tmpdir(), 'ailab-bdd-preflight-'));
  try {
    const file = join(dir, 'req.md');
    writeFileSync(file, '- [ ] (live) DeepSeek API key\n', 'utf8');
    const result = runCli(['--file', file, '--live']);
    assert.equal(result.status, 2);
    assert.match(result.stderr, /line 1: \(live\) DeepSeek API key/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test('CLI exits 2 for an empty checklist', () => {
  const dir = mkdtempSync(join(tmpdir(), 'ailab-bdd-preflight-'));
  try {
    const file = join(dir, 'req.md');
    writeFileSync(file, '   \n', 'utf8');
    const result = runCli(['--file', file]);
    assert.equal(result.status, 2);
    assert.match(result.stderr, /is empty/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
