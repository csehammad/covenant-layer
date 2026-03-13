#!/usr/bin/env python3
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List


ROOT = Path(__file__).resolve().parents[2]
FRAMEWORK_DIR = ROOT / "framework"
DEMO_DIR = ROOT / "demo"
MANIFEST_DIR = DEMO_DIR / "manifests"
ARTIFACTS_DIR = DEMO_DIR / "artifacts"


def run_cli(args: List[str]) -> Dict[str, Any]:
    cmd = ["cargo", "run", "--quiet", "--"] + args
    result = subprocess.run(
        cmd,
        cwd=FRAMEWORK_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"Command failed ({' '.join(cmd)}):\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}"
        )
    text = result.stdout.strip()
    if not text:
        return {}
    try:
        return json.loads(text)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Invalid JSON output for {' '.join(cmd)}:\n{text}") from exc


def env_required(key: str) -> str:
    value = os.getenv(key, "").strip()
    if not value:
        raise RuntimeError(f"Missing required env var: {key}")
    return value


def main() -> int:
    raw_tx_file = os.getenv("DEMO_RAW_TX_FILE", str(DEMO_DIR / "fixtures" / "raw-tx.template.json"))
    with open(raw_tx_file, "r", encoding="utf-8") as f:
        raw_tx_map = json.load(f)

    network = env_required("DEMO_NETWORK")
    rpc_url = env_required("DEMO_RPC_URL")
    contract = env_required("DEMO_REGISTRY_CONTRACT")
    stake_provider = os.getenv("DEMO_STAKE_PROVIDER", "500")
    stake_agent = os.getenv("DEMO_STAKE_AGENT", "500")

    participants = [
        {
            "participant_id": "agent.demo.orchestrator",
            "manifest": MANIFEST_DIR / "agent-service.demo.yaml",
            "owner": env_required("DEMO_AGENT_OWNER"),
            "stake": stake_agent,
        },
        {
            "participant_id": "provider.demo.alpha",
            "manifest": MANIFEST_DIR / "provider-alpha.demo.yaml",
            "owner": env_required("DEMO_PROVIDER_ALPHA_OWNER"),
            "stake": stake_provider,
        },
        {
            "participant_id": "provider.demo.bravo",
            "manifest": MANIFEST_DIR / "provider-bravo.demo.yaml",
            "owner": env_required("DEMO_PROVIDER_BRAVO_OWNER"),
            "stake": stake_provider,
        },
        {
            "participant_id": "provider.demo.charlie",
            "manifest": MANIFEST_DIR / "provider-charlie.demo.yaml",
            "owner": env_required("DEMO_PROVIDER_CHARLIE_OWNER"),
            "stake": stake_provider,
        },
    ]

    ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
    run_cli(["init"])

    artifact: Dict[str, Any] = {
        "mode": "chain",
        "network": network,
        "rpc_url": rpc_url,
        "contract": contract,
        "participants": [],
    }

    for p in participants:
        participant_id = p["participant_id"]
        manifest_path = str(p["manifest"])
        txs = raw_tx_map.get(participant_id, {})
        for needed in [
            "register",
            "identity_verified",
            "conformance_attestation",
            "conformance_passed",
            "probation",
            "active",
        ]:
            if not txs.get(needed):
                raise RuntimeError(f"Missing raw tx for {participant_id}.{needed} in {raw_tx_file}")

        steps: List[Dict[str, Any]] = []

        steps.append(
            {
                "step": "conformance_check",
                "output": run_cli(["conformance", "--manifest", manifest_path]),
            }
        )
        steps.append({"step": "sign_manifest", "output": run_cli(["sign", "--manifest", manifest_path])})
        steps.append(
            {
                "step": "verify_signature",
                "output": run_cli(["verify-signature", "--manifest", manifest_path]),
            }
        )
        steps.append(
            {
                "step": "register",
                "output": run_cli(
                    [
                        "register",
                        "--manifest",
                        manifest_path,
                        "--owner",
                        p["owner"],
                        "--stake",
                        str(p["stake"]),
                        "--mode",
                        "chain",
                        "--network",
                        network,
                        "--rpc-url",
                        rpc_url,
                        "--contract",
                        contract,
                        "--raw-tx",
                        txs["register"],
                    ]
                ),
            }
        )
        steps.append(
            {
                "step": "transition_identity_verified",
                "output": run_cli(
                    [
                        "transition",
                        "--participant-id",
                        participant_id,
                        "--to-state",
                        "identity_verified",
                        "--reason",
                        "demo identity verification",
                        "--mode",
                        "chain",
                        "--network",
                        network,
                        "--rpc-url",
                        rpc_url,
                        "--contract",
                        contract,
                        "--raw-tx",
                        txs["identity_verified"],
                    ]
                ),
            }
        )
        steps.append(
            {
                "step": "attach_conformance_attestation",
                "output": run_cli(
                    [
                        "conformance",
                        "--manifest",
                        manifest_path,
                        "--attach",
                        "--mode",
                        "chain",
                        "--network",
                        network,
                        "--rpc-url",
                        rpc_url,
                        "--contract",
                        contract,
                        "--raw-tx",
                        txs["conformance_attestation"],
                    ]
                ),
            }
        )
        steps.append(
            {
                "step": "transition_conformance_passed",
                "output": run_cli(
                    [
                        "transition",
                        "--participant-id",
                        participant_id,
                        "--to-state",
                        "conformance_passed",
                        "--reason",
                        "demo conformance passed",
                        "--mode",
                        "chain",
                        "--network",
                        network,
                        "--rpc-url",
                        rpc_url,
                        "--contract",
                        contract,
                        "--raw-tx",
                        txs["conformance_passed"],
                    ]
                ),
            }
        )
        steps.append(
            {
                "step": "transition_probation",
                "output": run_cli(
                    [
                        "transition",
                        "--participant-id",
                        participant_id,
                        "--to-state",
                        "probation",
                        "--reason",
                        "demo probation enabled",
                        "--mode",
                        "chain",
                        "--network",
                        network,
                        "--rpc-url",
                        rpc_url,
                        "--contract",
                        contract,
                        "--raw-tx",
                        txs["probation"],
                    ]
                ),
            }
        )
        steps.append(
            {
                "step": "transition_active",
                "output": run_cli(
                    [
                        "transition",
                        "--participant-id",
                        participant_id,
                        "--to-state",
                        "active",
                        "--reason",
                        "demo activation",
                        "--mode",
                        "chain",
                        "--network",
                        network,
                        "--rpc-url",
                        rpc_url,
                        "--contract",
                        contract,
                        "--raw-tx",
                        txs["active"],
                    ]
                ),
            }
        )
        steps.append(
            {
                "step": "status_chain",
                "output": run_cli(
                    [
                        "status",
                        "--source",
                        "chain",
                        "--network",
                        network,
                        "--rpc-url",
                        rpc_url,
                        "--contract",
                        contract,
                        "--participant-id",
                        participant_id,
                    ]
                ),
            }
        )
        artifact["participants"].append({"participant_id": participant_id, "steps": steps})

    out = ARTIFACTS_DIR / "onboarding.demo.json"
    out.write_text(json.dumps(artifact, indent=2), encoding="utf-8")
    print(json.dumps({"ok": True, "artifact": str(out)}, indent=2))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2))
        raise
