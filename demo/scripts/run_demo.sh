#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

echo "==> Running chain-only onboarding flow"
python3 "${SCRIPT_DIR}/run_chain_onboarding.py"

echo "==> Running objective/offer/acceptance flow"
python3 "${SCRIPT_DIR}/run_objective_flow.py"

echo "==> Building combined demo artifact"
python3 "${SCRIPT_DIR}/build_demo_artifact.py"

echo "==> Demo complete. Artifact: ${ROOT_DIR}/demo/artifacts/demo.full.json"
