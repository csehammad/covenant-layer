# Covenant Framework (Rust)

End-to-end framework for participant registration and onboarding with:
- local mode for deterministic development
- chain mode for EVM L2 testnet-ready artifacts
- YAML manifest + detached signature workflow
- versioned conformance profile (`conformance_profile: onboarding-v1`)
- onboarding lifecycle transitions and conformance attestations

## Core commands

```bash
cd framework
cargo run -- init
cargo run -- sign --manifest provider-manifest.template.yaml
cargo run -- verify-signature --manifest provider-manifest.template.yaml
```

### Local mode registration

```bash
cargo run -- register \
  --manifest provider-manifest.template.yaml \
  --owner 0xabc123 \
  --stake 500 \
  --mode local
```

### Chain mode registration (testnet)

```bash
cargo run -- register \
  --manifest provider-manifest.template.yaml \
  --owner 0xabc123 \
  --stake 500 \
  --mode chain \
  --network base_sepolia \
  --raw-tx 0xSIGNED_TRANSACTION_HEX
```

### Full end-to-end onboarding

```bash
cargo run -- e2e \
  --manifest provider-manifest.template.yaml \
  --owner 0xabc123 \
  --stake 500 \
  --mode chain \
  --network base_sepolia \
  --register-raw-tx 0xSIGNED_REGISTER_TX \
  --transition-raw-tx 0xSIGNED_TRANSITION_TX
```

### Chain-backed status

```bash
cargo run -- status --source chain --network base_sepolia
cargo run -- status --source chain --network base_sepolia --participant-id provider.example.travel
```

### Dispute lifecycle (v0.2.0)

Open and resolve disputes over accepted commitments in local or chain mode.
Full evidence stays off-chain; only digests and lifecycle transitions are
anchored on-chain. See [`runbook-dispute.md`](runbook-dispute.md) for details.

```bash
# Seed an accepted commitment (source of truth for enforcement)
cargo run -- commitment-record \
  --commitment-id cmt_01... \
  --accepted-by agent.example \
  --provider-id provider.example \
  --assigned-verifier verifier.example \
  --settlement-authority settlement.example \
  --settlement-state fulfilled

# Open a dispute (evidence file lists {type, digest} references)
cargo run -- dispute-open \
  --commitment-id cmt_01... \
  --reason terms_mismatch \
  --evidence dispute-evidence.json \
  --raised-by agent.example \
  --response-deadline 2026-08-06T12:00:00Z

cargo run -- dispute-status --dispute-id dsp_01...

# Resolve (assigned verifier or settlement authority only)
cargo run -- dispute-resolve \
  --dispute-id dsp_01... \
  --decision refunded \
  --resolution resolution.json \
  --resolved-by verifier.example
```

Enforced invariants: only a party to the commitment can open; the commitment
must be in a disputable state; the same dispute request cannot be replayed;
evidence is immutable after the response deadline; only the assigned verifier or
settlement authority can resolve; a resolved dispute cannot be reopened. Fixed
reason codes: `terms_mismatch`, `missing_evidence`, `late_fulfillment`,
`duplicate_charge`, `non_delivery`.

### Validation suite

```bash
cargo run -- validate-e2e \
  --manifest provider-manifest.template.yaml \
  --owner 0xabc123
```

### Local test run

```bash
cargo test
```

### Makefile shortcuts

```bash
make test-rust
make test-solidity
make test-all
make conformance-all
make deploy-testnet
```

### Solidity contract test run (Foundry)

```bash
forge install foundry-rs/forge-std
forge build
forge test
```

## Project files

- `networks.yaml` - supported network definitions
- `registry-contract-interface.md` - EVM contract interface and events
- `contracts/ProviderRegistry.sol` - reference on-chain registry contract
- `contracts/DisputeRegistry.sol` - reference on-chain dispute registry contract
- `foundry.toml` - Foundry project configuration
- `script/DeployProviderRegistry.s.sol` - Foundry deployment script
- `script/DeployDisputeRegistry.s.sol` - Foundry dispute registry deployment script
- `scripts/deploy_registry.sh` - shell helper for deployment
- `runbook-dispute.md` - dispute open/resolve runbook (local + chain)
- `runbook-deployer.md` - contract deployment and ownership runbook
- `governance-model.md` - ownership and upgrade control model
- `runbook-provider.md` - provider self-registration runbook
- `runbook-broker.md` - broker self-registration runbook
- `runbook-settlement-service.md` - settlement service self-registration runbook
- `runbook-verifier.md` - verifier self-registration runbook
- `runbook-agent-service.md` - agent service self-registration runbook
- `runbook-broker-verification.md` - broker eligibility verification runbook
- `conformance-profiles/onboarding-v1.md` - normative onboarding conformance profile
- `provider-manifest.template.yaml` - participant manifest template
- `examples/provider.emirates.candidate.yaml` - candidate onboarding manifest example
- `examples/broker.template.yaml` - broker manifest template
- `examples/settlement-service.template.yaml` - settlement service manifest template
- `examples/verifier.template.yaml` - verifier manifest template
- `examples/agent-service.template.yaml` - agent service manifest template

## Runtime notes

- Chain mode fetches live chain context (`eth_chainId`, `eth_blockNumber`).
- Chain mode requires signed transaction input and submits via `eth_sendRawTransaction`.
- Without `--raw-tx`, chain mode fails by design.
- State mirror is stored at `state/registry.json`.
- End-to-end chain-only demo assets and runbook are in `../demo/README.md`.

## CI

GitHub Actions runs framework CI automatically on `push` and `pull_request` for `framework/**`:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test --all`
- `forge build`
- `forge test`
