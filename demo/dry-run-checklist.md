# Base Sepolia Dry-Run Checklist

Use this checklist to validate the chain-only demo end-to-end.

## Prerequisites

- [ ] `demo/fixtures/chain-config.env` is populated with real network and owner addresses.
- [ ] `demo/fixtures/raw-tx.json` is populated with signed raw tx hex for each required step.
- [ ] Registry contract address is deployed and set in env.
- [ ] Wallets have enough ETH on Base Sepolia for all transactions.

## Execution

- [ ] `source demo/fixtures/chain-config.env` executed with exported vars.
- [ ] `DEMO_RAW_TX_FILE` points to `demo/fixtures/raw-tx.json`.
- [ ] `./demo/scripts/run_demo.sh` completes with no command failures.

## Expected outputs

- [ ] `demo/artifacts/onboarding.demo.json` exists and includes 4 participants.
- [ ] All participant final states are `active`.
- [ ] `demo/artifacts/objective-flow.demo.json` exists with 3 scored offers.
- [ ] `demo/artifacts/objective-flow.demo.json` includes `agent_reasoning_complete` event.
- [ ] `selection_method` is `llm_reasoning` (if API key set) or `deterministic` (if not).
- [ ] If LLM enabled: `agent_reasoning` block contains `model`, `reasoning`, `ranked_offers`, `confidence`.
- [ ] `demo/artifacts/objective-flow.demo.json` includes `offer_accepted`.
- [ ] `demo/artifacts/objective-flow.demo.json` includes `evidence_recorded`.
- [ ] `demo/artifacts/objective-flow.demo.json` includes `settlement_recorded`.
- [ ] `demo/artifacts/demo.full.json` exists and combines onboarding + objective flow.

## Post-run validation

- [ ] `python3 demo/scripts/validate_demo.py` returns `VALIDATION_OK`.
- [ ] `docs/demo-e2e-timeline.html` renders onboarding, offer comparison, and event trail.

## Pass/Fail gates

- **PASS**: every checklist item is checked.
- **FAIL**: any missing participant activation, missing event, or missing artifact.
