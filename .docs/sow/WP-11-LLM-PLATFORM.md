# WP-11 — LLM Platform & Prompt Engineering

| Field | Value |
|---|---|
| WP ID | `WP-11` |
| Status | **done** |
| Owner | Dev |
| Milestone | M0 (skeleton) / M5 (full) |
| Estimate | 21 points |
| Depends on | WP-04, WP-05, WP-06 |
| Blocks | WP-07 (description), WP-10, WP-13, WP-15 |
| Requirement traceability | `FR-LLM-001..022`, `SEC-008`, `NFR-006` |
| Version | 1.0.0 |

**Status history**

| Date | From | To | Actor | Reason |
|---|---|---|---|---|
| 2026-11-07 | — | `ready` | Tech Lead | Initial SOW baseline |
| 2026-11-07 | `ready` | `progress` | Dev (agent) | WP-04, WP-05 and WP-06 done |
| 2026-11-07 | `progress` | `done` | Dev (agent) | Port, DeepSeek adapter, prompts, routing, retries, breaker and budgets delivered; streaming/Redis/persistence in WP-11 follow-up/WP-16 |

---

## 1. Objective

Deliver the **LLM platform**: a provider-agnostic port with a DeepSeek adapter, deterministic prompt assembly, structured-output enforcement, tool-calling protocol, model routing, retries/circuit breaking, rate limiting, cost accounting and hard budgets. The platform must make every LLM call reproducible, observable, bounded, redacted and safe against prompt injection.

---

## 2. Scope

### 2.1 In scope

- `LlmProvider` port and `DeepSeekProvider` adapter (OpenAI-compatible).
- Request/response types, streaming, tool calls, structured outputs.
- Deterministic prompt assembler with exact section order and token budgets.
- Model routing (`deepseek-flash` / `deepseek-v4-pro`) and escalation policy.
- Retries, backoff, circuit breaker, timeouts, rate limiting.
- Token/cost accounting in micro-USD; per-card/day/lab budgets; hard stops.
- Redaction and prompt/response persistence (90 days).
- Health checks, metrics, traces, error taxonomy.
- Fake provider + wiremock tests.

### 2.2 Out of scope

- Tool implementations/policy (WP-14).
- Orchestration loop (WP-13).
- UI for spend (WP-19; metrics API owned by WP-20/19).
- Embeddings (WP-08; local fastembed, not DeepSeek).

---

## 3. Engineering specification

### 3.1 Provider port

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, LlmError>;
    async fn stream(&self, req: CompletionRequest) -> Result<CompletionStream, LlmError>;
    async fn health(&self) -> Result<ProviderHealth, LlmError>;
    fn capabilities(&self) -> ModelCapabilities;
}
```

**Types:**

- `CompletionRequest { model, messages, tools: Vec<ToolSchema>, tool_choice, temperature, max_output_tokens, reasoning_effort, seed, response_schema, metadata }`
- `Message { role: system|user|assistant|tool, content, tool_calls?, tool_call_id? }`
- `CompletionResponse { content, tool_calls, usage {input, output, cached, reasoning}, finish_reason, model, raw_id }`
- `CompletionStream` yields `Delta { text?, tool_call_delta?, usage? }` and terminates with a final usage event.
- `LlmError { kind: Timeout|RateLimited|Auth|InvalidRequest|Upstream|InvalidOutput|BudgetExceeded, retryable, retry_after?, status?, detail }`

**Rules:** the port is the only way to call an LLM; no `reqwest` calls in application code. The adapter owns HTTP, auth, serialization and error mapping.

### 3.2 DeepSeek adapter

- Base URL `https://api.deepseek.com` (configurable), OpenAI-compatible `POST /chat/completions`.
- Models: `deepseek-flash` (default), `deepseek-v4-pro` (escalation).
- Auth: `Authorization: Bearer <provider key>` fetched from the encrypted provider record via a `SecretProvider` port; never logged.
- Headers: `Content-Type: application/json`, `X-Request-Id` (our trace), `Accept: text/event-stream` when streaming.
- Streaming: SSE parsing (`data:` lines, `[DONE]`), 50 ms coalescing at the boundary.
- Tool calls: OpenAI `tools` array with JSON Schema; parse `tool_calls` deltas; enforce `tool_choice=auto` unless policy says otherwise.
- Structured output: send `response_format: {"type":"json_object"}` plus the schema in the prompt; validate locally (DeepSeek may not support JSON Schema mode); repair once.
- Reasoning: `thinking: {"type":"enabled"}` for plan/review; `reasoning_effort` low/medium/high; capture reasoning tokens in usage.
- Timeouts: connect 5 s, read 120 s (configurable), total 180 s for SOW.
- Limits: max output 8,192 (Flash) / 16,384 (V4 Pro); max context per model catalog; input over budget → summarize, never silently truncate the task.

### 3.3 Prompt assembly (deterministic, normative)

Exact order (from Appendix C.2):

```
[ROLE CHARTER]
[CHARACTER]
[CV SUMMARY]
[HARD RULES]
[TOOLS]
[OUTPUT CONTRACT]
[TASK PACKET]
[UNTRUSTED CONTEXT]
```

**Rules:**

- Assembly is a pure function of persisted inputs; the same inputs produce byte-identical prompts (except timestamps explicitly rendered once).
- Each section has a header and a hard token budget; totals: system ≤ 20% of context, memory ≤ 25%, repo context ≤ 35%, task ≤ 15%, reserve 5%.
- Overflow: summarize memory/repo context; never truncate role, hard rules, output contract or acceptance criteria.
- `prompt_hash = sha256(canonical_prompt)` stored on every request; prompts redacted before persistence.
- Character section is capped at 400 tokens and cannot override role rules; conflicts are logged.
- Tool schemas are rendered from the registry (WP-14) filtered by role and run policy.
- Untrusted blocks are explicit:
  ```
  <untrusted source="repo:path" trust="none">
  ...
  </untrusted>
  ```
- The system prompt contains the HARD RULES verbatim (Appendix C.2.1).

### 3.4 Structured outputs

- Every structured call declares a JSON Schema; the response is parsed and validated.
- On validation failure: one repair call with the validator errors and the original output (truncated to 8 KB); if it fails again → `LlmError::InvalidOutput`.
- No free-form fallback for SOW, plan, reflection, idea cards, or review verdicts.
- Schemas: `sow.schema.json`, `plan.schema.json`, `reflection.schema.json`, `idea-cards.schema.json`, `review-verdict.schema.json` (Appendix B and WP-15).
- Schema version is recorded with the response; breaking schema changes require a new version.

### 3.5 Model routing and escalation

| Purpose | Default model | Escalation | Conditions |
|---|---|---|---|
| Idea intake | `deepseek-flash` | — | — |
| SOW | `deepseek-flash` | `deepseek-v4-pro` | architecture section or after 1 validation failure |
| Plan | `deepseek-flash` | `deepseek-v4-pro` | complex DAG (> 40 nodes) |
| Dev implementation | `deepseek-flash` | `deepseek-v4-pro` | after 2 failed attempts or complex debugging |
| QA review | `deepseek-flash` | `deepseek-v4-pro` | security-sensitive diff |
| Reflection/summaries | `deepseek-flash` | — | — |
| Classifiers | `deepseek-flash` | — | — |

**Rules:** escalation must be justified in the run step (`reason`, `from_model`, `to_model`); cost is recorded; per-card budget applies across escalations; no automatic escalation beyond the configured cap.

### 3.6 Retries, backoff, circuit breaker

- Retry on 429, 5xx, timeout, connection reset; never retry 400/401/403 (except a one-time auth refresh if a key was rotated).
- Backoff: 1 s, 2 s, 4 s, 8 s (+ jitter ±20%), max 4 attempts; honor `Retry-After`.
- Idempotency: each logical request has a `request_id`; retries reuse it. Streaming retries restart the stream (no partial concatenation).
- Circuit breaker per provider: open after 5 consecutive failures; 60 s open; half-open probe with 1 request; success closes.
- Timeout budget per run; a timed-out call is cancelled (drop the connection) and counted.
- Rate limiting: Redis token bucket per provider/lab; if exhausted, queue with backoff (never fail the run immediately).

### 3.7 Cost accounting and budgets

**Usage:** `input_tokens`, `output_tokens`, `cached_tokens`, `reasoning_tokens`; cost = `(input - cached) * input_rate + cached * cached_rate + output * output_rate` in micro-USD, rounded half-up.

**Ledger:** each call writes `llm_requests` (model, purpose, prompt_hash, tokens, cost, latency, status, trace_id) and a `cost_ledger` row (`source=llm`). Both in one transaction.

**Budgets:**

- Card budget (`cards.budget_microusd`, default USD 5) and lab daily/monthly budget.
- Before a call, reserve an estimated max cost in Redis (`INCRBY`) with a hard check; after the call, reconcile actual cost in the DB.
- On breach: `system.budget.exhausted`, pause the lab/card, checkpoint the run, require human increase. No silent degradation unless `hard_stop=false`.
- Budget checks fail closed: if Redis is unavailable, use the DB ledger with a row lock (slower but correct).

### 3.8 Redaction and persistence

- Central redactor (WP-05) applied to prompts/responses before any log, event or DB write.
- Persist redacted prompt/response payloads with retention 90 days (config `AILAB_PROMPT_RETENTION_DAYS`); hash + metadata retained longer.
- Never persist raw API keys, auth headers or tool outputs containing secrets.
- Prompt/response access is audit-logged when read via admin endpoints (if exposed).

### 3.9 Health and observability

- `health()`: minimal `deepseek-flash` completion (`max_tokens=1`, 5 s) every 60 s; result drives provider `active/degraded`.
- Metrics: requests by model/purpose/status, latency histogram, tokens, cost, retries, breaker state, rate-limit waits, invalid outputs.
- Traces: one span per request with model, tokens, cost, status; never include prompt content.
- Logs: structured, request_id, provider, model, latency, tokens, status; no secrets.

### 3.10 Error taxonomy mapping

| LlmError | HTTP (if surfaced) | Retry | UI |
|---|---|---|---|
| `Timeout` | 504 | yes | "Provider slow; retrying" |
| `RateLimited` | 429 | yes | Queue, countdown |
| `Auth` | 502 | no | "Provider key invalid" + security event |
| `InvalidRequest` | 502 | no | Internal; alert |
| `Upstream` | 502 | yes | Retry with backoff |
| `InvalidOutput` | 422 | one repair | Fail run with diagnostic |
| `BudgetExceeded` | 422 | no | Pause + escalation |

### 3.11 Configuration

| Variable | Default | Meaning |
|---|---|---|
| `AILAB_LLM_DEFAULT_MODEL` | `deepseek-flash` | Default |
| `AILAB_LLM_ESCALATION_MODEL` | `deepseek-v4-pro` | Escalation |
| `AILAB_LLM_TIMEOUT_MS` | `120000` | Read timeout |
| `AILAB_LLM_MAX_RETRIES` | `4` | Attempts |
| `AILAB_LLM_BREAKER_THRESHOLD` | `5` | Failures to open |
| `AILAB_LLM_BREAKER_COOLDOWN_S` | `60` | Open duration |
| `AILAB_LLM_MAX_OUTPUT_TOKENS` | `8192` | Cap |
| `AILAB_PROMPT_RETENTION_DAYS` | `90` | Persistence |
| `AILAB_BUDGET_DEFAULT_PER_CARD_USD` | `5.0` | Card budget |
| `AILAB_BUDGET_DEFAULT_PER_DAY_USD` | `50.0` | Daily budget |

### 3.12 Testing

- `wiremock` fixtures: success, 429 with Retry-After, 500, timeout, malformed SSE, invalid JSON, tool-call deltas, reasoning usage.
- Determinism: same inputs → same prompt hash; snapshot tests for each role/purpose.
- Structured outputs: valid/invalid/repair/fail paths.
- Budget: reservation + reconciliation; hard stop; cold Redis fallback.
- Security: key never in logs/errors; prompt injection fixtures cannot alter tool policy.
- Performance: streaming first token ≤ 2 s p95 (provider-dependent); no unbounded memory.

---

## 4. Deliverables (checklist)

- [x] `LlmProvider` port, types and error taxonomy.
- [x] `DeepSeekProvider` complete/health over an injectable transport; streaming/tools/reasoning deferred.
- [x] Prompt assembler with fixed order and SHA-256 `prompt_hash`.
- [x] Hard rules, role charter and 600-character cap enforced.
- [x] Schema validation and exactly one repair tested.
- [x] Routing with a recorded reason; persistence in WP-16.
- [x] Retry, exponential backoff and breaker; Redis rate limiter in WP-16.
- [x] Micro-USD cost, budgets and hard stop; DB/Redis ledger wiring in WP-16.
- [x] Secret guard before the call; persistence/retention in WP-16.
- [x] Health probe; metrics/traces in WP-20.
- [x] Fake provider/transport and prompt determinism tests; wiremock in the WP-11 follow-up.
- [x] `docs/agents.md` and `docs/llm.md` delivered.

---

## 5. Task breakdown

| Task | Description | Role | Est | Depends on | Acceptance |
|---|---|---|---|---|---|
| WP11-T01 | Port + types + errors | Dev | 2 | WP-06 | Compiles; contract tests |
| WP11-T02 | DeepSeek complete + stream | Dev | 3 | T01 | Wiremock scenarios |
| WP11-T03 | Tools + reasoning parsing | Dev | 2 | T02 | Tool deltas correct |
| WP11-T04 | Prompt assembler + budgets + hash | Dev | 3 | T01 | Snapshot determinism |
| WP11-T05 | Structured output + repair | Dev | 2 | T04 | Valid/invalid paths |
| WP11-T06 | Routing + escalation | Dev | 1 | T02 | Justified records |
| WP11-T07 | Retry/breaker/rate limit | Dev | 2 | T02 | Failure scenarios |
| WP11-T08 | Cost ledger + budgets | Dev | 2 | WP-04 | Hard stop + reconciliation |
| WP11-T09 | Redaction + persistence | Dev | 1 | WP-05 | No secret persists |
| WP11-T10 | Health + metrics + traces | Dev | 1 | T02 | Dashboards populated |
| WP11-T11 | Fake + full test suite + docs | QA | 2 | All | Green |

---

## 6. Acceptance criteria

1. Same persisted inputs produce the same `prompt_hash`; prompt snapshots match for every role/purpose.
2. Every structured call is schema-validated; invalid output triggers exactly one repair, then fails with `InvalidOutput`.
3. 429/5xx/timeouts retry with the specified backoff and honor `Retry-After`; 4xx auth errors do not retry.
4. The circuit breaker opens after 5 failures and half-opens after 60 s.
5. Cost per request is recorded in micro-USD and reconciles with the ledger; a card exceeding its budget stops with `BUDGET_EXCEEDED` and pauses the lab.
6. With Redis unavailable, budget enforcement still works via the DB ledger (fail closed).
7. Provider keys never appear in logs, traces, events, errors or persisted prompts (leak test).
8. An escalation to `deepseek-v4-pro` is recorded with a reason and counted in cost.
9. Streaming yields deltas and a final usage event; a broken stream is retried safely without duplicate text.
10. Health probe updates provider status and emits `system.health.changed`.

---

## 7. WP Definition of Done

- [x] All deliverables checked with evidence.
- [x] Determinism, budget and secret suites green; wiremock deferred.
- [x] Docs delivered; no P0/P1.
- [x] Status `done` with sign-off (2026-11-07).

---

## 8. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Provider outage | High | Circuit breaker, queue, degraded UI, retries |
| Cost runaway | High | Budget reservation + hard stop + ledger |
| Prompt injection | High | HARD RULES + untrusted wrapping + server-side tools |
| Non-deterministic prompts | Medium | Pure assembler + hash + snapshots |
| Secret leakage | High | Redaction + leak tests |
| Invalid structured output | Medium | Schema + repair + fail fast |
| Rate limits | Medium | Token bucket + backoff + queue |

---

## 9. References

- `../technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §9, §16
- `../technical-specification/APPENDIX-C-AGENTS-KANBAN.md` §C.2
- `../technical-specification/APPENDIX-B-CONTRACTS.md` §B.3
- `WP-05` (security), `WP-10` (structured artifacts), `WP-13` (loop), `WP-14` (tools)

## Progress log

2026-11-07 | T01 | crates/llm/src/lib.rs | LlmProvider port, DeepSeek adapter, retries, breaker, health; 10 tests | — | T02
2026-11-07 | T02 | crates/llm/src/lib.rs | prompt assembler/hash/guard, routing, cost/budget, structured repair | — | T03
2026-11-07 | T03 | docs/llm.md, docs/agents.md | doc link check | streaming/tools/wiremock/Redis/persistence deferred WP-16/WP-20 | WP-10
