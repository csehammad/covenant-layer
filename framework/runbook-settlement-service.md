# Settlement Service Runbook

Use this runbook to onboard a settlement service.

## Steps

1. Validate settlement manifest:

```bash
cargo run -- conformance --manifest examples/settlement-service.template.yaml
```

2. Sign and verify:

```bash
cargo run -- sign --manifest examples/settlement-service.template.yaml
cargo run -- verify-signature --manifest examples/settlement-service.template.yaml
```

3. Register in chain mode with signed transaction:

```bash
cargo run -- register \
  --manifest examples/settlement-service.template.yaml \
  --owner 0xSETTLEMENT_OWNER \
  --stake 500 \
  --mode chain \
  --network base_sepolia \
  --raw-tx 0xSIGNED_REGISTER_TX
```

4. Transition to `active` only after settlement/dispute conformance evidence is attached.

## Settlement hardening requirements

Before `probation -> active`, verify:

- `evidence_link_integrity_ref` is declared and audited
- `finality_wait_seconds` is positive and policy-compliant
- `replay_protection: true` is enforced in runtime and settlement handlers
