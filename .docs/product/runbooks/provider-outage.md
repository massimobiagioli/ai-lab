# Provider outage

- **Detect:** LLM/SCM errors above 1%, `AilabHighErrorRate` or provider health `invalid`.
- **Check:** provider status, credential validity, the last rotation, the model routing fallback.
- **Mitigate:** route to the fallback model, pause the affected agent, rotate the credential if invalid, requeue the failed runs with backoff.
- **Escalate:** Tech Lead; pause the lab if no provider is healthy for 30 min.
