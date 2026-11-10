# AI-Lab LLM Platform

| Field | Value |
|---|---|
| Document | `.docs/product/llm.md` |
| Version | 1.0.0 |
| Status | Accepted |
| Owner | Dev |
| Related | `.docs/product/agents.md`, `.docs/product/security.md`, `crates/llm/src/lib.rs` |

DeepSeek is the only v1 provider, behind an `LlmProvider` port. The platform owns routing, prompt assembly, structured outputs, retries, circuit breaking, cost accounting and budgets.

## 1. Port and types

- `LlmProvider::complete(&LlmRequest) -> Result<LlmResponse, LlmError>` and `health()`.
- `LlmRequest`: model, system, user, temperature, `json_mode`.
- `LlmResponse`: content, `Usage { tokens_in, tokens_out }`, model.
- `LlmError`: Transport, RateLimited, Auth, Timeout, InvalidOutput, BudgetExceeded, CircuitOpen, SecretDetected, Validation.

The `DeepSeekProvider` adapter builds an OpenAI-compatible request and runs over an injectable `HttpTransport`, so tests use a fake transport with no network.

## 2. Routing

| Condition | Model | Reason |
|---|---|---|
| default | `deepseek-flash` | default |
| confidence < 0.5 or prior failure | `deepseek-v4-pro` | escalation: low confidence or prior failure |

The route reason is part of the decision and is recorded by the caller (WP-16 persistence).

## 3. Prompt assembly

The assembler renders, in this order (Appendix C.2):

1. `ROLE CHARTER`
2. `CHARACTER` (≤ 600 characters)
3. `CV SUMMARY`
4. `HARD RULES`
5. `TOOLS`
6. `OUTPUT CONTRACT`
7. `TASK PACKET`
8. `UNTRUSTED CONTEXT` blocks wrapped as `<untrusted label="...">...</untrusted>`

- Untrusted blocks escape a closing `</untrusted>` so content cannot break the wrapper.
- A prompt containing a secret prefix (`sk-`, `ghp_`, `github_pat_`, `AKIA`, `AIza`, `xoxb-`), a PEM block, `AILAB_MASTER_KEY=` or `password=` is rejected with `SecretDetected`.
- `prompt_hash` is the SHA-256 of the rendered system and user prompts; the same inputs produce the same hash.

## 4. Structured outputs

Structured calls are parsed into a typed value and validated. An invalid output triggers **exactly one** repair request; a second invalid output fails with `InvalidOutput`. `DraftCard` validates title length, acceptance criteria, deliverables, estimate and priority.

## 5. Retries, backoff and circuit breaker

- 429 and 5xx are retried up to 4 attempts; 401/403 are not retried.
- Backoff is exponential (1 s, 2 s, 4 s, 8 s) with ±20% jitter and honors `Retry-After`.
- The circuit opens after 5 consecutive failures and half-opens after 60 s.
- The adapter records success/failure in the breaker and exposes `health()`.

## 6. Cost and budgets

- `ModelPrice` holds input/output micro-USD per million tokens.
- Cost = `(tokens_in * input_price + tokens_out * output_price) / 1_000_000` micro-USD, integer arithmetic.
- `Budget` tracks per-card and per-day spend; exceeding either returns `BudgetExceeded` and the caller stops the run.
- The DB ledger remains authoritative; Redis is a fast path (WP-16).

## 7. Tests

`scripts/run-rust.sh cargo test -p ailab-llm` covers prompt determinism and wrapping, secret detection, the character cap, routing, budget hard stop, micro-USD cost, retry/backoff/breaker, DeepSeek retry on 429, no retry on 401, structured repair once, and failure after one repair.

## 8. Deferred to consumer WPs

- Streaming, tools and reasoning deltas: WP-11 follow-up.
- Real HTTP transport and wiremock suite: WP-11 follow-up.
- Prompt/response persistence, retention and call records: WP-16.
- Redis rate limiter and cold-cache budget fallback: WP-16.
- Metrics and traces: WP-20.

## 9. References

- `.docs/sow/WP-11-LLM-PLATFORM.md`
- `.docs/technical-specification/AI-LAB-TECHNICAL-SPECIFICATION.md` §9
- `crates/llm/src/lib.rs`
