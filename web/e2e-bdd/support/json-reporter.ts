import { mkdirSync, writeFileSync } from 'node:fs';
import { basename, join, resolve } from 'node:path';
import type { FullResult, Reporter, TestCase, TestResult } from '@playwright/test/reporter';
import { redact } from './redact';

interface ResultRow {
  id: string;
  title: string;
  area: string;
  priority: string;
  status: 'passed' | 'failed' | 'skipped';
  duration_ms: number;
  error?: string;
  tags: string[];
  artifact_paths: string[];
  location: string;
}

function scenarioId(title: string): string {
  return /E2E-\d{3}/.exec(title)?.[0] ?? 'E2E-UNKNOWN';
}

function normalizeTag(tag: string): string {
  return tag.replace(/^@/, '');
}

function priority(tags: readonly string[]): string {
  return tags.map(normalizeTag).find((tag) => /^p[0-2]$/.test(tag)) ?? 'unset';
}

export default class JsonReporter implements Reporter {
  private readonly rows: ResultRow[] = [];
  private startedAt = Date.now();

  onBegin(): void {
    this.startedAt = Date.now();
  }

  onTestEnd(test: TestCase, result: TestResult): void {
    const tags = test.tags.map(normalizeTag);
    const status: ResultRow['status'] =
      result.status === 'passed' ? 'passed' : result.status === 'skipped' ? 'skipped' : 'failed';
    const artifacts = result.attachments
      .map((attachment) => attachment.path)
      .filter((path): path is string => Boolean(path));
    this.rows.push({
      id: scenarioId(test.title),
      title: test.title,
      area: basename(test.location.file, '.feature'),
      priority: priority(tags),
      status,
      duration_ms: result.duration,
      ...(result.error?.message ? { error: redact(result.error.message) } : {}),
      tags,
      artifact_paths: artifacts,
      location: `${test.location.file}:${test.location.line}`,
    });
  }

  async onEnd(result: FullResult): Promise<void> {
    const dir = resolve(__dirname, 'reports');
    mkdirSync(dir, { recursive: true });
    const passed = this.rows.filter((row) => row.status === 'passed').length;
    const failed = this.rows.filter((row) => row.status === 'failed').length;
    const skipped = this.rows.filter((row) => row.status === 'skipped').length;
    const manual = this.rows.filter((row) => row.tags.includes('manual')).length;
    const live = this.rows.filter((row) => row.tags.includes('live')).length;
    const failedIds = this.rows.filter((row) => row.status === 'failed').map((row) => row.id);
    const payload = {
      ok: failed === 0,
      runStatus: result.status,
      startedAt: new Date(this.startedAt).toISOString(),
      durationMs: Date.now() - this.startedAt,
      summary: {
        total: this.rows.length,
        passed,
        failed,
        skipped,
        manual,
        live,
      },
      failedIds,
      results: this.rows.sort((a, b) => a.id.localeCompare(b.id)),
    };
    writeFileSync(join(dir, 'results.json'), `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
    process.stdout.write(
      `BDD summary: total=${this.rows.length} passed=${passed} failed=${failed} skipped=${skipped} manual=${manual} live=${live}\n`,
    );
    if (failedIds.length > 0) process.stdout.write(`BDD failed: ${failedIds.join(', ')}\n`);
  }
}
