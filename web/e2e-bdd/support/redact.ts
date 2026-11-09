const SECRET_PATTERNS: readonly RegExp[] = [
  /\bsk-[A-Za-z0-9_-]{8,}\b/g,
  /\bghp_[A-Za-z0-9]{8,}\b/g,
  /\bgithub_pat_[A-Za-z0-9_]{8,}\b/g,
  /\bAKIA[0-9A-Z]{16}\b/g,
  /\bAIza[0-9A-Za-z_-]{20,}\b/g,
  /\bxox[baprs]-[A-Za-z0-9-]{8,}\b/g,
  /\b(Bearer\s+)[A-Za-z0-9._~+/=-]{8,}/gi,
  /("?(?:api[_-]?key|token|password|authorization|secret)"?\s*[:=]\s*)("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|[^\s,;]+)/gi,
  /-----BEGIN [A-Z ]+ PRIVATE KEY-----[\s\S]*?-----END [A-Z ]+ PRIVATE KEY-----/g,
];

export function redact(value: string): string {
  let result = value;
  for (const pattern of SECRET_PATTERNS) {
    result = result.replace(pattern, (match, prefix: string | undefined) => {
      if (prefix && /^"?[a-z_]+"?\s*[:=]/i.test(prefix)) return `${prefix}[REDACTED]`;
      if (/^Bearer\s+/i.test(prefix ?? '')) return `${prefix ?? ''}[REDACTED]`;
      if (match.startsWith('-----BEGIN')) return '[REDACTED PRIVATE KEY]';
      return '[REDACTED]';
    });
  }
  return result;
}

export function redactUnknown(value: unknown): unknown {
  if (typeof value === 'string') return redact(value);
  if (Array.isArray(value)) return value.map(redactUnknown);
  if (value && typeof value === 'object') {
    const result: Record<string, unknown> = {};
    for (const [key, entry] of Object.entries(value)) {
      result[key] = redactUnknown(entry);
    }
    return result;
  }
  return value;
}
