#!/usr/bin/env node
// List and resolve the six BDD suites.
//
// Usage:
//   node scripts/bdd-suites.mjs --list
//   node scripts/bdd-suites.mjs --suite core --json
//   node scripts/bdd-suites.mjs --up-to delivery --json
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { grepFor, loadSuites, selectSuites, suiteCounts, validateSuites } from '../.tools/lib/suites.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const DEFAULT_ROOT = resolve(HERE, '..');

function usage() {
  process.stdout.write(`Usage: node scripts/bdd-suites.mjs [options]

Options:
  --list          List the six suites and their scenario counts.
  --suite ID      Select one suite.
  --up-to ID      Select every suite up to and including ID.
  --json          Emit machine-readable JSON for the selection.
  --root DIR      Repository root (default: ${DEFAULT_ROOT}).
  --help          Show this help.

Exit codes:
  0  OK
  2  invalid manifest or selection
`);
}

function parseArgs(argv) {
  const args = { root: DEFAULT_ROOT, list: false, json: false };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--help' || arg === '-h') return { help: true, args };
    if (arg === '--list') args.list = true;
    else if (arg === '--json') args.json = true;
    else if (arg === '--suite') args.suite = argv[++i];
    else if (arg === '--up-to') args.upTo = argv[++i];
    else if (arg === '--root') args.root = resolve(argv[++i] ?? '');
    else return { error: `unknown argument: ${arg}`, args };
  }
  if (args.suite && args.upTo) return { error: 'choose either --suite or --up-to, not both', args };
  if (!args.list && !args.suite && !args.upTo) args.list = true;
  return { args };
}

function main() {
  const parsed = parseArgs(process.argv.slice(2));
  if (parsed.help) {
    usage();
    return;
  }
  if (parsed.error) {
    process.stderr.write(`bdd-suites: ${parsed.error}\n`);
    process.exit(2);
  }
  const { root, list, json, suite, upTo } = parsed.args;
  const suites = loadSuites(root);
  const validation = validateSuites(suites);
  if (validation.errors.length > 0) {
    process.stderr.write(`bdd-suites: invalid suite manifest:\n${validation.errors.map((error) => `  - ${error}`).join('\n')}\n`);
    process.exit(2);
  }

  const counts = suiteCounts(suites);
  if (list && !json) {
    process.stdout.write('Order  Suite        Scenarios  Title\n');
    for (const entry of counts) {
      process.stdout.write(`${String(entry.order).padEnd(6)} ${entry.id.padEnd(12)} ${String(entry.count).padEnd(10)} ${entry.title}\n`);
    }
    process.stdout.write(`\nTotal: ${counts.reduce((sum, entry) => sum + entry.count, 0)} scenarios across ${counts.length} suites.\n`);
    return;
  }

  let selected;
  try {
    selected = selectSuites(suites, { suite, upTo });
  } catch (error) {
    process.stderr.write(`bdd-suites: ${error instanceof Error ? error.message : String(error)}\n`);
    process.exit(2);
  }
  const selectedIds = selected.map((entry) => entry.id);
  const isAll = !suite && !upTo;
  const payload = {
    suites: selectedIds,
    counts: Object.fromEntries(counts.map((entry) => [entry.id, entry.count])),
    total: selected.reduce((sum, entry) => sum + (counts.find((count) => count.id === entry.id)?.count ?? 0), 0),
    grep: grepFor(selected, isAll),
    all: isAll,
  };

  if (json) {
    process.stdout.write(`${JSON.stringify(payload, null, 2)}\n`);
    return;
  }
  process.stdout.write(`Suites: ${selectedIds.join(', ')}\n`);
  process.stdout.write(`Scenarios: ${payload.total}\n`);
  process.stdout.write(`Playwright grep: ${payload.grep || '(all)'}\n`);
}

main();
