/** Stable id generator that works in secure and non-secure browser contexts. */
export function createId(prefix = ''): string {
  const random =
    typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function'
      ? crypto.randomUUID()
      : `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
  return prefix === '' ? random : `${prefix}-${random}`;
}
