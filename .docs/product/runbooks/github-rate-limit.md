# GitHub rate limit

- **Detect:** SCM 403/429 with rate-limit headers, `AilabHighErrorRate` on SCM calls.
- **Check:** `parse_rate_limit` headers, the token scopes, the webhook backlog.
- **Mitigate:** back off until the reset, reduce the concurrency, use the secondary token if configured, replay the webhook deliveries.
- **Escalate:** Tech Lead if the limit persists for more than 1 hour.
