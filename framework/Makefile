SHELL := /bin/bash

.PHONY: help test-rust test-solidity test-all lint-rust fmt-rust build-rust build-solidity \
        install-forge-std deploy-testnet validate-e2e init sign verify conformance-all

help:
	@echo "Available targets:"
	@echo "  make test-rust         - Run Rust test suite"
	@echo "  make test-solidity     - Run Foundry contract tests"
	@echo "  make test-all          - Run Rust + Solidity tests"
	@echo "  make lint-rust         - Run clippy with warnings as errors"
	@echo "  make fmt-rust          - Check rustfmt formatting"
	@echo "  make build-rust        - Compile Rust framework"
	@echo "  make build-solidity    - Compile Solidity contracts with Foundry"
	@echo "  make install-forge-std - Install forge-std library"
	@echo "  make deploy-testnet    - Deploy ProviderRegistry using .env variables"
	@echo "  make init              - Initialize local framework state store"
	@echo "  make sign              - Sign default provider manifest"
	@echo "  make verify            - Verify default provider manifest signature"
	@echo "  make validate-e2e      - Run framework validation suite"
	@echo "  make conformance-all   - Run conformance checks for all templates"

fmt-rust:
	cargo fmt --all -- --check

lint-rust:
	cargo clippy --all-targets --all-features -- -D warnings

build-rust:
	cargo check

test-rust:
	cargo test --all

install-forge-std:
	forge install foundry-rs/forge-std

build-solidity:
	forge build

test-solidity:
	forge test

test-all: fmt-rust lint-rust test-rust test-solidity

deploy-testnet:
	./scripts/deploy_registry.sh

init:
	cargo run -- init

sign:
	cargo run -- sign --manifest provider-manifest.template.yaml

verify:
	cargo run -- verify-signature --manifest provider-manifest.template.yaml

validate-e2e:
	cargo run -- validate-e2e --manifest provider-manifest.template.yaml --owner 0xabc123

conformance-all:
	cargo run -- conformance --manifest provider-manifest.template.yaml
	cargo run -- conformance --manifest examples/provider.emirates.candidate.yaml
	cargo run -- conformance --manifest examples/broker.template.yaml
	cargo run -- conformance --manifest examples/settlement-service.template.yaml
	cargo run -- conformance --manifest examples/verifier.template.yaml
	cargo run -- conformance --manifest examples/agent-service.template.yaml
