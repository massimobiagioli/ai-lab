#!/usr/bin/env node
// Manual preflight gate for the BDD suite.
//
// It reads .docs/e2e/req.md and refuses to start the suite when a manual
// prerequisite is still unchecked. The checklist is minimal: only operations a
// human must perform are allowed in the file.
//
// Usage:
//   node scripts/bdd-preflight.mjs [--file PATH] [--json]
import { existsSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const DEFAULT_FILE = resolve(HERE, '..', '.docs', 'e2e', 'req.md');

const CHECKBOX_RE = /^\s*-\s*\[([ xX!-])\]\s*(.*)$/;

export function parseChecklist(markdown) {
  const lines = markdown.split(/\r?\n/);
  const items = [];
  let inFence = false;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (/^\s*```/.test(line)) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    const match = CHECKBOX_RE.exec(line);
    if (!match) continue;
    const marker = match[1];
    const text = match[2].trim();
    items.push({
      line: index + 1,
      marker,
      text,
      checked: marker === 'x' || marker === 'X',
      na: /\bN\/A\b/i.test(text),
    });
  }
  return items;
}

export function evaluateChecklist(markdown) {
  const items = parseChecklist(markdown);
  const missing = items.filter((item) => !item.checked && !item.na);
  return {
    ok: missing.length === 0,
    total: items.length,
    checked: items.filter((item) => item.checked || item.na).length,
    missing: missing.map((item) => ({ line: item.line, text: item.text, marker: item.marker })),
  };
}

export function formatHuman(result, file) {
  if (result.ok) return `BDD preflight: ready (${result.checked}/${result.total} manual setup items resolved in ${file}).\n`;
  const lines = [
    `BDD suite not started: missing manual setup in ${file}.`,
    'Missing items:',
    ...result.missing.map((item) => `  - line ${item.line}: ${item.text}`),
    `Fix the checklist in ${file} and run \`make bdd\` again. The suite was not started.`,
    '',
  ];
  return lines.join('\n');
}

function usage() {
  process.stdout.write(`Usage: node scripts/bdd-preflight.mjs [options]

Options:
  --file PATH   Checklist file (default: .docs/e2e/req.md).
  --json        Emit machine-readable JSON.
  --help        Show this help.

Rules:
  - every "- [ ]" item in the checklist is a missing prerequisite;
  - "- [x]" is satisfied;
  - an item marked N/A is satisfied;
  - checkboxes inside fenced code blocks are ignored.

Exit codes:
  0  ready
  2  missing manual setup or unreadable checklist
`);
}

function main() {
  const args = process.argv.slice(2);
  let file = DEFAULT_FILE;
  let json = false;
  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === '--help' || arg === '-h') {
      usage();
      return;
    }
    if (arg === '--json') json = true;
    else if (arg === '--file') file = resolve(args[++i] ?? '');
    else {
      process.stderr.write(`bdd-preflight: unknown argument: ${arg}\n`);
      process.exit(2);
    }
  }

  if (!existsSync(file)) {
    const message = `BDD suite not started: ${file} is missing or unreadable.`;
    if (json) process.stdout.write(`${JSON.stringify({ ok: false, file, error: 'missing-file', missing: [] }, null, 2)}\n`);
    else process.stderr.write(`${message}\n`);
    process.exit(2);
  }

  const markdown = readFileSync(file, 'utf8');
  if (markdown.trim() === '') {
    const message = `BDD suite not started: ${file} is empty.`;
    if (json) process.stdout.write(`${JSON.stringify({ ok: false, file, error: 'empty-file', missing: [] }, null, 2)}\n`);
    else process.stderr.write(`${message}\n`);
    process.exit(2);
  }

  const result = evaluateChecklist(markdown);
  if (json) {
    process.stdout.write(`${JSON.stringify({ ...result, file }, null, 2)}\n`);
  } else if (result.ok) {
    process.stdout.write(formatHuman(result, file));
  } else {
    process.stderr.write(formatHuman(result, file));
  }
  process.exit(result.ok ? 0 : 2);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main();
}
