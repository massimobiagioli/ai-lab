## Card

[{CARD_KEY}] {CARD_TITLE}
{PR_URL_TO_CARD?} · SOW: {SOW_LINK}

## Summary

- What changed and why (3–6 bullets)

## Acceptance criteria

- [ ] AC-1: {text} — evidence: {command/test/file}
- [ ] AC-2: {text} — evidence: {command/test/file}

## Changes

- {path} — {what and why}

## Verification

- [ ] `cargo fmt --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo test`
- [ ] `npm run lint && npm test`
- [ ] `scripts/harness-lint.sh`
- [ ] `scripts/sow-lint.sh`
- [ ] secret scan
- Coverage: {number}% (new code)

## Risks / rollback

- {risk} → {mitigation}
- Rollback: {how}

## AI-Lab trace

Run: {run_id}
Agent: {agent_name} ({agent_id})
Model: {model}
Attempt: {n}

---

Reviewer: confirm reviewer ≠ author and merger ≠ author (ADR-0011). A self-review or self-merge is rejected with `422 SEPARATION_OF_DUTIES`. Required status contexts are listed in `.github/required-checks.txt`.
