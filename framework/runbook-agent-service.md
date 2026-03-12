# Agent Service Runbook

Use this runbook to onboard an agent service participant.

## Steps

1. Validate agent service manifest:

```bash
cargo run -- conformance --manifest examples/agent-service.template.yaml
```

2. Sign and verify:

```bash
cargo run -- sign --manifest examples/agent-service.template.yaml
cargo run -- verify-signature --manifest examples/agent-service.template.yaml
```

3. Register in chain mode:

```bash
cargo run -- register \
  --manifest examples/agent-service.template.yaml \
  --owner 0xAGENT_OWNER \
  --stake 500 \
  --mode chain \
  --network base_sepolia \
  --raw-tx 0xSIGNED_REGISTER_TX
```

4. Ensure `approval_gating_required: true` and authority policy references are validated before active routing.

## Agent activation gate requirements

Before moving an agent service from `probation` to `active`, governance should verify:

- `authority_proof_method` is explicitly declared
- `approval_reference_format` is explicitly declared
- `grant_revocation_check_seconds` is positive and operationally enforced
- `max_authority_ttl_seconds` is positive and enforced in runtime policy
- conformance attestation is attached to registry events
