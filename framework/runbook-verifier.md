# Verifier Runbook

Use this runbook to onboard a verifier participant.

## Steps

1. Validate verifier manifest:

```bash
cargo run -- conformance --manifest examples/verifier.template.yaml
```

2. Sign and verify:

```bash
cargo run -- sign --manifest examples/verifier.template.yaml
cargo run -- verify-signature --manifest examples/verifier.template.yaml
```

3. Register on testnet:

```bash
cargo run -- register \
  --manifest examples/verifier.template.yaml \
  --owner 0xVERIFIER_OWNER \
  --stake 500 \
  --mode chain \
  --network base_sepolia \
  --raw-tx 0xSIGNED_REGISTER_TX
```

4. Promote from `probation` to `active` only after verification domain and replay-protection checks are independently audited.

## Verifier hardening requirements

Before `probation -> active`, verify:

- `replay_protection: true` is enforced in verification runtime
- `verification_sla_seconds` is positive and monitored
- `decision_policy_ref` is published and auditable
