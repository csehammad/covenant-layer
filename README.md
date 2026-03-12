# Covenant Layer

Covenant Layer is an open model for shifting agent systems from tool orchestration to outcome coordination.

The core claim is simple:

Modern agents are probabilistic systems being forced to operate deterministic software. That mismatch is where a lot of today’s fragility comes from. The long-term fix is not to keep teaching agents how to click better. It is to make software increasingly expose commitments about outcomes instead of only low-level procedures.

## Core idea

In the old model, the agent:
- calls tools
- runs scripts
- uses browser automation
- stitches workflows together
- directly carries too much execution burden

In the Covenant Layer model:
- the user states an objective
- the agent carries authority
- a broker or exchange routes that objective
- providers return offers they are willing to stand behind
- the agent compares and accepts one
- the provider fulfills the outcome
- the network records evidence and settlement

The key shift is from procedures to commitments.

## Project components

- **Covenant Layer**: the architectural shift from procedural interfaces to outcome coordination.
- **Covenant Network**: the participant ecosystem that forms around shared commitment semantics.
- **Covenant Protocol**: the formal rulebook for objectives, authority, offers, acceptance, evidence, settlement, and disputes.
- **Covenant Framework (Rust)**: implementation scaffold for participant onboarding, conformance, and registry operations.

## Repository layout (current)

- `spec/` - protocol specification set
  - `00-intro.md` to `04-examples.md` define core context, protocol, security, and examples
  - `05-wire-integrity.md` defines envelope, canonicalization/signature, replay/idempotency, and error taxonomy
  - `06-governance-dispute-upgrades.md` defines governance authority, dispute lifecycle, and upgrade controls
  - `07-compatibility-matrix.md` defines protocol/profile compatibility and migration behavior
  - `schemas/protocol-envelope.v1.schema.json` provides machine-readable schema baseline
- `framework/` - Rust implementation for onboarding and registry workflow
  - manifest signing and verification
  - actor-specific conformance checks (provider/broker/settlement/verifier/agent service)
  - onboarding state transitions and eligibility model
  - local mode and EVM L2 chain mode
  - runbooks, governance model, contract interface, and examples
- `docs/` - architecture, mental model, network model, security rationale, and planning artifacts
- `diagrams/` - text diagrams for old-vs-new model, layered architecture, and lifecycle
- `article.md` - Part I essay (concept thesis)
- `article-part-ii.md` - Part II essay (implementation and under-the-hood details)
- `GOVERNANCE.md`, `CONTRIBUTING.md`, `LICENSE.md`, `LICENSES/` - project governance/contribution/license artifacts

## Framework quick start

From repo root:

```bash
cd framework
cargo run -- init
cargo run -- conformance --manifest provider-manifest.template.yaml
cargo run -- sign --manifest provider-manifest.template.yaml
cargo run -- verify-signature --manifest provider-manifest.template.yaml
```

Common checks:

```bash
cd framework
make test-rust
make conformance-all
```

## Status

This is an active draft + implementation repository.

Current state:

- protocol model is documented with formal extension chapters
- onboarding and conformance pipeline is implemented in Rust
- actor templates and runbooks are included for end-to-end testnet registration workflows
- security/governance/dispute/compatibility baselines are explicitly documented

The goal remains the same: make the model concrete enough to critique, implement, and improve.