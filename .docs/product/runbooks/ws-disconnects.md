# WebSocket disconnects

- **Detect:** reconnects, `resync` events, `AilabNoTraffic`, the health pill offline.
- **Check:** the WS endpoint, the proxy timeouts, the event backlog, the client last-seq.
- **Mitigate:** let the client resume with `?since`; replay `/events?after`; restart the hub only after draining; verify the heartbeat.
- **Escalate:** Tech Lead if the clients cannot resume.
