#!/usr/bin/env node
// Generate deterministic Gherkin feature files from .docs/e2e/BDD.md.
//
// Usage:
//   node .tools/gen-bdd-features.mjs --check [--root DIR] [--expected N]
//   node .tools/gen-bdd-features.mjs --write [--root DIR] [--expected N]
//
// --check  fail when the generated files are missing, stale or unexpected.
// --write  regenerate the feature files and remove stale generated files.
// --root   repository root (default: the parent of this tool).
// --expected expected scenario count (default: 480).
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildFeatures, summarize } from './lib/bdd-parser.mjs';
import { loadSuites, suiteCounts, validateSuites } from './lib/suites.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const DEFAULT_ROOT = resolve(HERE, '..');
const MANIFEST = '.generated.json';

function usage() {
  process.stdout.write(`Usage: node .tools/gen-bdd-features.mjs --check|--write [options]

Options:
  --check           Verify that generated features match .docs/e2e/BDD.md.
  --write           Regenerate the feature files.
  --root DIR        Repository root (default: ${DEFAULT_ROOT}).
  --expected N      Expected scenario count (default: 480).
  --help            Show this help.

Exit codes:
  0  OK
  1  Drift, invalid source or invalid arguments
`);
}

function parseArgs(argv) {
  const args = { root: DEFAULT_ROOT, expected: 480, mode: null };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--help' || arg === '-h') return { help: true, args };
    if (arg === '--check') args.mode = 'check';
    else if (arg === '--write') args.mode = 'write';
    else if (arg === '--root') args.root = resolve(argv[++i] ?? '');
    else if (arg === '--expected') args.expected = Number(argv[++i]);
    else return { error: `unknown argument: ${arg}`, args };
  }
  if (!args.mode) return { error: 'one of --check or --write is required', args };
  if (!Number.isInteger(args.expected) || args.expected < 1) return { error: '--expected must be a positive integer', args };
  return { args };
}

function fail(message) {
  process.stderr.write(`gen-bdd-features: ${message}\n`);
  process.exit(1);
}

function listFeatureFiles(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).filter((name) => name.endsWith('.feature')).sort();
}

function readManifest(dir) {
  const file = join(dir, MANIFEST);
  if (!existsSync(file)) return [];
  try {
    const parsed = JSON.parse(readFileSync(file, 'utf8'));
    return Array.isArray(parsed.files) ? parsed.files : [];
  } catch {
    return [];
  }
}

function compare(dir, files) {
  const existing = new Set(listFeatureFiles(dir));
  const expected = new Set(files.keys());
  const missing = [...expected].filter((name) => !existing.has(name));
  const stale = [...existing].filter((name) => !expected.has(name));
  const changed = [...expected].filter((name) => {
    const target = join(dir, name);
    return existsSync(target) && readFileSync(target, 'utf8') !== files.get(name);
  });
  return { missing, stale, changed };
}

function main() {
  const parsed = parseArgs(process.argv.slice(2));
  if (parsed.help) {
    usage();
    return;
  }
  if (parsed.error) fail(parsed.error);
  const { root, expected, mode } = parsed.args;
  const sourceFile = join(root, '.docs', 'e2e', 'BDD.md');
  const outputDir = join(root, 'web', 'e2e-bdd', 'features');

  if (!existsSync(sourceFile)) fail(`missing source: ${sourceFile}`);
  const markdown = readFileSync(sourceFile, 'utf8');
  const suites = loadSuites(root);
  const suiteValidation = validateSuites(suites, expected);
  if (suiteValidation.errors.length > 0) {
    process.stderr.write(`gen-bdd-features: invalid suite manifest:\n${suiteValidation.errors.map((e) => `  - ${e}`).join('\n')}\n`);
    process.exit(1);
  }
  const { files, scenarios, errors, expectedCount } = buildFeatures(markdown, {
    expectedCount: expected,
    suiteByScenario: suiteValidation.suiteByScenario,
  });

  if (errors.length > 0) {
    process.stderr.write(`gen-bdd-features: invalid BDD source:\n${errors.map((e) => `  - ${e}`).join('\n')}\n`);
    process.exit(1);
  }
  const counts = suiteCounts(suites);

  if (mode === 'write') {
    mkdirSync(outputDir, { recursive: true });
    const previous = readManifest(outputDir);
    const nextNames = new Set(files.keys());
    for (const name of previous) {
      if (!nextNames.has(name)) {
        const target = join(outputDir, name);
        if (existsSync(target)) rmSync(target);
      }
    }
    for (const [name, content] of files) {
      writeFileSync(join(outputDir, name), content, 'utf8');
    }
    writeFileSync(join(outputDir, MANIFEST), `${JSON.stringify({ expected: expectedCount, files: [...nextNames].sort(), summary: summarize(files, scenarios) }, null, 2)}\n`, 'utf8');
    const summary = summarize(files, scenarios);
    const suiteSummary = counts.map((entry) => `${entry.id}:${entry.count}`).join(', ');
    process.stdout.write(`gen-bdd-features: wrote ${summary.features} feature files with ${summary.scenarios} scenarios (${suiteSummary})\n`);
    return;
  }

  const diff = compare(outputDir, files);
  if (diff.missing.length || diff.stale.length || diff.changed.length) {
    process.stderr.write('gen-bdd-features: generated features are out of date:\n');
    for (const name of diff.missing) process.stderr.write(`  missing: ${name}\n`);
    for (const name of diff.stale) process.stderr.write(`  stale:   ${name}\n`);
    for (const name of diff.changed) process.stderr.write(`  changed: ${name}\n`);
    process.stderr.write('Run: node .tools/gen-bdd-features.mjs --write\n');
    process.exit(1);
  }
  const summary = summarize(files, scenarios);
  const suiteSummary = counts.map((entry) => `${entry.id}:${entry.count}`).join(', ');
  process.stdout.write(`gen-bdd-features: OK (${summary.features} features, ${summary.scenarios} scenarios, ${suiteSummary}, hash ${summary.hash.slice(0, 12)})\n`);
}

main();
