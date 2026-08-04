# Runbook: Dispute Lifecycle

This runbook covers opening and resolving disputes over accepted commitments in
both local and chain modes. Full evidence stays off-chain; only digests, an
evidence root, and lifecycle transitions are anchored on-chain.

## Roles

- **Opener**: a party to the accepted commitment (the accepting agent/user or
  the fulfilling provider).
- **Assigned verifier**: reviews signed evidence from both parties.
- **Settlement authority**: records the final settlement transition. Bound to
  the commitment via `--settlement-authority` (local mode) or the
  `DisputeRegistry` governance address (chain mode).

## Preconditions

- The commitment has been recorded (`commitment-record`) and is in a disputable
  settlement state (`fulfilled`, `failed`, or `settled`).
- Evidence files (`dispute-evidence.json`, `resolution.json`) list evidence
  references as `{ "type": ..., "digest": "sha256:..." }`.

Example `dispute-evidence.json`:

```json
{
  "statement": "Booked cabin does not match the accepted offer",
  "evidence_refs": [
    { "type": "accepted_offer", "digest": "sha256:..." },
    { "type": "provider_receipt", "digest": "sha256:..." }
  ]
}
```

Example `resolution.json`:

```json
{
  "resolution_evidence_refs": [
    { "type": "verifier_report", "digest": "sha256:..." }
  ]
}
```

## Local mode

```bash
cargo run -- commitment-record \
  --commitment-id cmt_01... \
  --accepted-by agent.example \
  --provider-id provider.example \
  --assigned-verifier verifier.example \
  --settlement-authority settlement.example \
  --settlement-state fulfilled

cargo run -- dispute-open \
  --commitment-id cmt_01... \
  --reason terms_mismatch \
  --evidence dispute-evidence.json \
  --raised-by agent.example \
  --response-deadline 2026-08-06T12:00:00Z

cargo run -- dispute-status --dispute-id dsp_01...

cargo run -- dispute-resolve \
  --dispute-id dsp_01... \
  --decision refunded \
  --resolution resolution.json \
  --resolved-by verifier.example
```

## Chain mode

Chain mode requires a deployed `DisputeRegistry` and signed raw transactions
(no simulated submissions). Deploy with `script/DeployDisputeRegistry.s.sol`.

```bash
cargo run -- dispute-open \
  --commitment-id cmt_01... \
  --reason terms_mismatch \
  --evidence dispute-evidence.json \
  --raised-by agent.example \
  --response-deadline 2026-08-06T12:00:00Z \
  --mode chain \
  --network base_sepolia \
  --contract 0xDISPUTE_REGISTRY \
  --raw-tx 0xSIGNED_OPEN_DISPUTE_TX

cargo run -- dispute-resolve \
  --dispute-id dsp_01... \
  --decision refunded \
  --resolution resolution.json \
  --resolved-by verifier.example \
  --mode chain \
  --network base_sepolia \
  --contract 0xDISPUTE_REGISTRY \
  --raw-tx 0xSIGNED_RESOLVE_DISPUTE_TX
```

## Enforced invariants

- only a party to the accepted commitment can open a dispute
- the commitment must be in a disputable settlement state
- the same dispute request cannot be replayed (deterministic `dispute_id`)
- dispute evidence is immutable after `response_deadline`
- only the assigned verifier or settlement authority can resolve
- a resolved dispute cannot be reopened under the same identifier

## Reason codes

`terms_mismatch`, `missing_evidence`, `late_fulfillment`, `duplicate_charge`,
`non_delivery`. The free-text `statement` is supplementary; settlement logic
does not interpret prose.
