import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const STEP_RE = /(Given|When|Then)\s*\(\s*(?:'((?:[^'\\]|\\.)*)'|"((?:[^"\\]|\\.)*)"|\/((?:[^/\\]|\\.)*)\/([gimsuy]*))\s*,/g;

export function cucumberToRegex(source) {
  let out = '';
  let index = 0;
  while (index < source.length) {
    const rest = source.slice(index);
    const token = /^\{(string|word|int|float|bool|)\}/.exec(rest);
    if (token) {
      const kind = token[1];
      if (kind === 'string') out += '(?:"([^"]*)"|\'([^\']*)\')';
      else if (kind === 'word') out += '(\\S+)';
      else if (kind === 'int') out += '(-?\\d+)';
      else if (kind === 'float') out += '(-?\\d+(?:\\.\\d+)?)';
      else if (kind === 'bool') out += '(true|false)';
      else out += '(.+?)';
      index += token[0].length;
      continue;
    }
    const char = source[index];
    out += /[.*+?^${}()|[\]\\]/.test(char) ? `\\${char}` : char;
    index += 1;
  }
  return out;
}

export function listStepFiles(dir) {
  if (!existsSync(dir)) return [];
  const out = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...listStepFiles(full));
    else if (entry.isFile() && entry.name.endsWith('.ts')) out.push(full);
  }
  return out.sort();
}

export function loadStepDefinitions(stepDir, options = {}) {
  const exclude = new Set(options.excludeFiles ?? []);
  const definitions = [];
  for (const file of listStepFiles(stepDir)) {
    const base = file.split('/').pop();
    if (base && exclude.has(base)) continue;
    const content = readFileSync(file, 'utf8');
    STEP_RE.lastIndex = 0;
    let match;
    while ((match = STEP_RE.exec(content)) !== null) {
      const keyword = match[1];
      const stringSource = match[2] ?? match[3];
      const regexSource = match[4];
      const flags = match[5] ?? '';
      let source;
      let regex;
      let kind;
      if (regexSource !== undefined) {
        kind = 'regex';
        source = regexSource;
        regex = new RegExp(source, flags);
      } else {
        kind = 'expression';
        source = stringSource ?? '';
        regex = new RegExp(`^${cucumberToRegex(source)}$`);
      }
      definitions.push({ file, keyword, kind, source, regex });
    }
  }
  return definitions;
}

export function resolveKeywords(steps) {
  let current = 'Given';
  return steps.map((step) => {
    if (step.keyword === 'Given' || step.keyword === 'When' || step.keyword === 'Then') current = step.keyword;
    return { ...step, resolvedKeyword: current };
  });
}

export function matchDefinitions(step, definitions) {
  return definitions.filter((definition) => definition.keyword === step.resolvedKeyword && definition.regex.test(step.text));
}

export function computeCoverage(scenarios, definitions) {
  const unmatchedSteps = [];
  const scenariosWithoutRealSteps = [];
  const matchedDefinitionIndexes = new Set();

  for (const scenario of scenarios) {
    const steps = resolveKeywords(scenario.steps);
    let realMatches = 0;
    for (const step of steps) {
      const matches = matchDefinitions(step, definitions);
      if (matches.length === 0) {
        unmatchedSteps.push({ id: scenario.id, keyword: step.resolvedKeyword, text: step.text });
      } else {
        realMatches += matches.length;
        for (const definition of matches) {
          const index = definitions.indexOf(definition);
          if (index >= 0) matchedDefinitionIndexes.add(index);
        }
      }
    }
    if (realMatches === 0) scenariosWithoutRealSteps.push(scenario.id);
  }

  return { unmatchedSteps, scenariosWithoutRealSteps, matchedDefinitionIndexes };
}

export function escapeForRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

export function stepKey(keyword, text) {
  return `${keyword}\u0000${text}`;
}
