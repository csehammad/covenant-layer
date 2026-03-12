# Broker Verification Runbook

Use this runbook to enforce participant eligibility from chain-backed state.

## Verification policy

- Route only to participants with state `active`.
- Reject `requested`, `identity_verified`, `conformance_passed`, `probation`, `restricted`, `revoked`.
- Log tx hash, block number, and contract metadata for audit.

## Commands

1. Pull all chain-sourced participants:

```bash
cargo run -- status --source chain --network base_sepolia
```

2. Pull specific participant:

```bash
cargo run -- status --source chain --network base_sepolia --participant-id provider.example.travel
```

## Gate check checklist

- participant exists in chain source output
- state equals `active`
- last update includes expected network and contract
- signature verified during registration flow
- no unresolved restriction or revocation event after active state

## Incident handling

- On suspected fraud:
  - move participant to `restricted` per governance authority
  - suspend routing immediately
  - require corrective attestation before probation re-entry
