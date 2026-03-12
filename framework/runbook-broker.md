# Broker Runbook

Use this runbook to onboard a broker as a first-class network participant.

## Steps

1. Validate broker manifest:

```bash
cargo run -- conformance --manifest examples/broker.template.yaml
```

2. Sign and verify:

```bash
cargo run -- sign --manifest examples/broker.template.yaml
cargo run -- verify-signature --manifest examples/broker.template.yaml
```

3. Register on chain:

```bash
cargo run -- register \
  --manifest examples/broker.template.yaml \
  --owner 0xBROKER_OWNER \
  --stake 500 \
  --mode chain \
  --network base_sepolia \
  --raw-tx 0xSIGNED_REGISTER_TX
```

4. Move through onboarding states (`identity_verified`, `conformance_passed`, `probation`, `active`) using `transition` with signed raw transactions.

## Broker hardening requirements

Before `probation -> active`, verify:

- `conflict_of_interest_policy_ref` is present and auditable
- `routing_audit_log_retention_days` is positive and meets policy minimum
- `fairness_review_interval_days` is positive and operationally scheduled
