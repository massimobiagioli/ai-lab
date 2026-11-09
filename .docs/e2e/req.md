# AI-Lab — Manual Setup Checklist for BDD E2E

Only one manual setup is required. Everything else is provisioned automatically by the BDD tooling.

- [ ] DeepSeek API key (`sk-...`, valid and funded). Run `make bdd-set-api-key` and enter the key at the hidden prompt. The command sends it to AI-Lab, stores it encrypted and shows only the last four characters. It is needed only for live BDD runs; the default suite uses the fake LLM harness.

No other manual setup is required.
