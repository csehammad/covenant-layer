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
- `foundry.toml` - Foundry project configuration
- `script/DeployProviderRegistry.s.sol` - Foundry deployment script
- `scripts/deploy_registry.sh` - shell helper for deployment
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
