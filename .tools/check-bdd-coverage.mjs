#!/usr/bin/env node
// Verify BDD coverage and drift for the Playwright-BDD suite.
//
// Usage:
//   node .tools/check-bdd-coverage.mjs [--root DIR] [--expected N] [--report-only] [--json]
//
// Structural checks always fail on a duplicate, missing or unexpected E2E id,
// on a stale generated feature and on a duplicate scenario. By default the
// command also fails when a scenario or step is not backed by a real step
// definition. Generated exact-match steps for the narrative scenarios count as
// definitions; --report-only exits 0 for a scaffold inventory.
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildFeatures, EXPECTED_SCENARIO_COUNT } from './lib/bdd-parser.mjs';
import { computeCoverage, loadStepDefinitions } from './lib/step-parser.mjs';
import { loadSuites, suiteCounts, validateSuites } from './lib/suites.mjs';

const HERE = dirname(fileURLToPath(import.meta.url));
const DEFAULT_ROOT = resolve(HERE, '..');
const GENERATED_STEPS = 'generated.steps.ts';

function usage() {
  process.stdout.write(`Usage: node .tools/check-bdd-coverage.mjs [options]

Options:
  --root DIR     Repository root (default: ${DEFAULT_ROOT}).
  --expected N   Expected scenario count (default: ${EXPECTED_SCENARIO_COUNT}).
  --report-only  Report pending steps but exit 0 (scaffold inventory).
  --strict       Fail when any scenario or step is not backed by a step definition (default).
  --json         Emit the machine-readable report to stdout.
  --help         Show this help.

Exit codes:
  0  structural checks pass and coverage is complete (or --report-only)
  1  structural failure or pending coverage
`);
}

function parseArgs(argv) {
  const args = { root: DEFAULT_ROOT, expected: EXPECTED_SCENARIO_COUNT, strict: true, json: false };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--help' || arg === '-h') return { help: true, args };
    if (arg === '--strict') args.strict = true;
    else if (arg === '--report-only') args.strict = false;
    else if (arg === '--json') args.json = true;
    else if (arg === '--root') args.root = resolve(argv[++i] ?? '');
    else if (arg === '--expected') args.expected = Number(argv[++i]);
    else return { error: `unknown argument: ${arg}`, args };
  }
  if (!Number.isInteger(args.expected) || args.expected < 1) return { error: '--expected must be a positive integer', args };
  return { args };
}

function fail(message) {
  process.stderr.write(`check-bdd-coverage: ${message}\n`);
  process.exit(1);
}

function featureFiles(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).filter((name) => name.endsWith('.feature')).sort();
}

function compareFeatures(dir, files) {
  const existing = new Set(featureFiles(dir));
  const expected = new Set(files.keys());
  return {
    missing: [...expected].filter((name) => !existing.has(name)),
    stale: [...existing].filter((name) => !expected.has(name)),
    changed: [...expected].filter((name) => existsSync(join(dir, name)) && readFileSync(join(dir, name), 'utf8') !== files.get(name)),
  };
}

function manualScenarioIds(root) {
  const file = join(root, 'web', 'e2e-bdd', 'manual-scenarios.md');
  if (!existsSync(file)) return { ids: new Set(), file };
  const content = readFileSync(file, 'utf8');
  return { ids: new Set([...content.matchAll(/E2E-\d{3}/g)].map((m) => m[0])), file };
}

function main() {
  const parsed = parseArgs(process.argv.slice(2));
  if (parsed.help) {
    usage();
    return;
  }
  if (parsed.error) fail(parsed.error);
  const { root, expected, strict, json } = parsed.args;
  const sourceFile = join(root, '.docs', 'e2e', 'BDD.md');
  const featureDir = join(root, 'web', 'e2e-bdd', 'features');
  const stepDir = join(root, 'web', 'e2e-bdd', 'steps');
  const reportDir = join(root, 'web', 'e2e-bdd', 'reports');

  if (!existsSync(sourceFile)) fail(`missing source: ${sourceFile}`);
  const markdown = readFileSync(sourceFile, 'utf8');
  const suites = loadSuites(root);
  const suiteValidation = validateSuites(suites, expected);
  const { files, features, scenarios, errors } = buildFeatures(markdown, {
    expectedCount: expected,
    suiteByScenario: suiteValidation.suiteByScenario,
  });

  const structural = [...errors, ...suiteValidation.errors];
  const diff = compareFeatures(featureDir, files);
  for (const name of diff.missing) structural.push(`missing generated feature: ${name}`);
  for (const name of diff.stale) structural.push(`stale generated feature: ${name}`);
  for (const name of diff.changed) structural.push(`stale generated content: ${name} (run gen-bdd-features --write)`);

  const allDefinitions = loadStepDefinitions(stepDir);
  const curatedDefinitions = loadStepDefinitions(stepDir, { excludeFiles: [GENERATED_STEPS] });
  const manual = manualScenarioIds(root);
  const coverage = computeCoverage(scenarios, allDefinitions);
  const curatedCoverage = computeCoverage(scenarios, curatedDefinitions);

  const unusedDefinitions = allDefinitions
    .filter((_definition, index) => !coverage.matchedDefinitionIndexes.has(index))
    .map((definition) => ({ file: definition.file, keyword: definition.keyword, source: definition.source }));

  const pendingScenarios = coverage.scenariosWithoutRealSteps.filter((id) => !manual.ids.has(id));
  const report = {
    ok: structural.length === 0 && (!strict || (coverage.unmatchedSteps.length === 0 && pendingScenarios.length === 0)),
    expected,
    scenarios: scenarios.length,
    features: features.length,
    definitions: allDefinitions.length,
    curatedDefinitions: curatedDefinitions.length,
    generatedDefinitions: allDefinitions.length - curatedDefinitions.length,
    suites: suiteCounts(suites),
    manualScenarios: [...manual.ids].sort(),
    unmatchedSteps: coverage.unmatchedSteps,
    pendingScenarios,
    curatedPendingScenarios: curatedCoverage.scenariosWithoutRealSteps.filter((id) => !manual.ids.has(id)),
    curatedPendingSteps: curatedCoverage.unmatchedSteps.length,
    unusedDefinitions,
    structuralErrors: structural,
    generated: Object.fromEntries([...files.keys()].sort().map((name) => [name, true])),
  };

  mkdirSync(reportDir, { recursive: true });
  writeFileSync(join(reportDir, 'coverage.json'), `${JSON.stringify(report, null, 2)}\n`, 'utf8');

  if (json) {
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
  } else {
    if (structural.length > 0) {
      process.stderr.write(`check-bdd-coverage: structural errors:\n${structural.map((e) => `  - ${e}`).join('\n')}\n`);
    }
    process.stdout.write(`check-bdd-coverage: ${scenarios.length} scenarios, ${features.length} features, ${allDefinitions.length} step definitions (${report.generatedDefinitions} generated).\n`);
    process.stdout.write(`  suites:            ${report.suites.map((suite) => `${suite.id}:${suite.count}`).join(', ')}\n`);
    process.stdout.write(`  pending scenarios: ${pendingScenarios.length}\n`);
    process.stdout.write(`  pending steps:     ${coverage.unmatchedSteps.length}\n`);
    process.stdout.write(`  curated pending:   ${report.curatedPendingScenarios.length} scenarios / ${report.curatedPendingSteps} steps\n`);
    process.stdout.write(`  unused definitions: ${unusedDefinitions.length}\n`);
    if (strict && !report.ok) {
      process.stderr.write('check-bdd-coverage: coverage incomplete; implement the pending steps or list the scenarios in web/e2e-bdd/manual-scenarios.md.\n');
    }
  }
  process.exit(report.ok ? 0 : 1);
}

main();
