import { chmodSync, copyFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { randomBytes } from 'node:crypto';
import { resolve } from 'node:path';

const root = resolve(process.env.AILAB_ROOT ?? process.cwd());
const envPath = resolve(root, '.env');
const examplePath = resolve(root, '.env.example');

if (!existsSync(envPath)) {
  if (!existsSync(examplePath)) {
    process.stderr.write(`generate-key: missing ${examplePath}; cannot create .env.\n`);
    process.exit(1);
  }
  copyFileSync(examplePath, envPath);
}

let content = readFileSync(envPath, 'utf8');

function validMasterKey(value) {
  if (!value || value.startsWith('replace-')) return false;
  try {
    return Buffer.from(value, 'base64').length === 32;
  } catch {
    return false;
  }
}

function validRunnerToken(value) {
  return /^[A-Za-z0-9._-]{32,}$/.test(value);
}

function existing(name) {
  const match = new RegExp(`^${name}=(.*)$`, 'm').exec(content);
  return match?.[1]?.trim() ?? '';
}

function upsert(name, value) {
  const line = `${name}=${value}`;
  const pattern = new RegExp(`^${name}=.*$`, 'm');
  if (pattern.test(content)) content = content.replace(pattern, line);
  else content = `${content.replace(/\s*$/, '')}\n${line}\n`;
  return line;
}

const master = existing('AILAB_MASTER_KEY');
const runner = existing('AILAB_RUNNER_TOKEN');

let masterStatus = 'already set';
if (!validMasterKey(master)) {
  upsert('AILAB_MASTER_KEY', randomBytes(32).toString('base64'));
  masterStatus = 'generated (32 bytes, base64)';
}

let runnerStatus = 'already set';
if (!validRunnerToken(runner)) {
  upsert('AILAB_RUNNER_TOKEN', randomBytes(32).toString('hex'));
  runnerStatus = 'generated (32 bytes, hex)';
}

writeFileSync(envPath, content, { mode: 0o600 });
chmodSync(envPath, 0o600);
process.stdout.write(`AILAB_MASTER_KEY: ${masterStatus}\n`);
process.stdout.write(`AILAB_RUNNER_TOKEN: ${runnerStatus}\n`);
process.stdout.write('.env updated; secrets are not printed.\n');
