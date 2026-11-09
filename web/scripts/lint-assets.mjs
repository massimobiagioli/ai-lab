// Fail the build when the generated pixel atlas is missing frames or a licence.
import { readFileSync } from 'node:fs';

const source = readFileSync(new URL('../src/lib/office/sprites.ts', import.meta.url), 'utf8');
const errors = [];

if (!source.includes("license: 'CC0-1.0'")) {
  errors.push('sprite manifest must declare the CC0-1.0 licence');
}
if (!source.includes("source: 'generated in-repo")) {
  errors.push('sprite manifest must declare the in-repo source');
}

const blocks = source.match(/pattern:\s*\[([\s\S]*?)\],\s*palette/g) ?? [];
const expectedSprites = ['idle', 'type', 'blocked', 'sleep'];
for (const name of expectedSprites) {
  if (!new RegExp(`\\b${name}:\\s*\\{`).test(source)) {
    errors.push(`missing sprite animation '${name}'`);
  }
}
if (blocks.length !== expectedSprites.length) {
  errors.push(`expected ${expectedSprites.length} sprite patterns, found ${blocks.length}`);
}
for (const block of blocks) {
  const rows = block.match(/'[^']*'/g) ?? [];
  if (rows.length !== 8) {
    errors.push('each sprite must have 8 rows');
  }
  for (const row of rows) {
    if (row.length !== 10) {
      errors.push(`sprite row ${row} must be 8 pixels plus quotes`);
    }
  }
}
for (const value of source.match(/'#[0-9a-fA-F]{6}'/g) ?? []) {
  if (value.length !== 9) {
    errors.push(`invalid palette colour ${value}`);
  }
}

if (errors.length > 0) {
  for (const error of errors) {
    console.error(`lint-assets: ${error}`);
  }
  process.exit(1);
}
console.log(`lint-assets: OK (${blocks.length} sprites)`);
