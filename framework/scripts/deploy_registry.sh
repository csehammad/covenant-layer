#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${RPC_URL:-}" ]]; then
  echo "RPC_URL is required"
  exit 1
fi

if [[ -z "${PRIVATE_KEY:-}" ]]; then
  echo "PRIVATE_KEY is required"
  exit 1
fi

if [[ -z "${GOVERNANCE_ADDRESS:-}" ]]; then
  echo "GOVERNANCE_ADDRESS is required"
  exit 1
fi

cd "$(dirname "$0")/.."

forge script script/DeployProviderRegistry.s.sol:DeployProviderRegistry \
  --rpc-url "$RPC_URL" \
  --private-key "$PRIVATE_KEY" \
  --broadcast
