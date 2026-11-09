# Stuck runs

- **Detect:** run age above the lease, `timed_out` runs, queue depth growth, no heartbeat.
- **Check:** `runs` heartbeat/lease, the worker process, the runner health.
- **Mitigate:** let the reaper time out and requeue; restart the worker; cancel the run if the sandbox is orphaned; confirm the label reaper removed containers.
- **Escalate:** Tech Lead if the queue does not drain within 30 min.
