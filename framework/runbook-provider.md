# Provider Runbook

Use this runbook to self-register a provider on testnet.

## Prerequisites

- Rust toolchain installed
- access to an EVM L2 RPC endpoint
- participant manifest YAML prepared
- detached signature file generated
- manifest sets `conformance_profile: onboarding-v1`

## Steps

1. Initialize state store:

```bash
cargo run -- init
```

2. Validate and sign manifest:

```bash
cargo run -- conformance --manifest provider-manifest.template.yaml
cargo run -- sign --manifest provider-manifest.template.yaml
cargo run -- verify-signature --manifest provider-manifest.template.yaml
```

3. Register in chain mode:

```bash
cargo run -- register \
  --manifest provider-manifest.template.yaml \
  --owner 0xabc123 \
  --stake 500 \
  --mode chain \
  --network base_sepolia \
  --raw-tx 0xSIGNED_REGISTER_TX
```

4. Advance onboarding states:

```bash
cargo run -- transition --participant-id provider.example.travel --to-state identity_verified --reason "identity validated" --mode chain --network base_sepolia --raw-tx 0xSIGNED_TX_1
cargo run -- transition --participant-id provider.example.travel --to-state conformance_passed --reason "conformance passed" --mode chain --network base_sepolia --raw-tx 0xSIGNED_TX_2
cargo run -- transition --participant-id provider.example.travel --to-state probation --reason "bounded routing" --mode chain --network base_sepolia --raw-tx 0xSIGNED_TX_3
cargo run -- transition --participant-id provider.example.travel --to-state active --reason "promotion threshold met" --mode chain --network base_sepolia --raw-tx 0xSIGNED_TX_4
```

5. Confirm chain-backed status:

```bash
cargo run -- status --source chain --network base_sepolia --participant-id provider.example.travel
```

## Notes

- Chain mode requires `--raw-tx` and submits via `eth_sendRawTransaction`.
- If `--raw-tx` is missing, the command fails.

## Provider hardening requirements

Before `probation -> active`, verify:

- `provider_policy.incident_response_policy_ref` is published and audited
- `provider_policy.data_protection_policy_ref` is published and audited
- `provider_policy.fulfillment_sla_seconds` is positive and operationally monitored
- `provider_policy.idempotency_window_seconds` is positive and enforced in fulfillment APIs
