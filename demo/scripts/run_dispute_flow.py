#!/usr/bin/env python3
"""Offline terms-mismatch dispute + refund resolution flow.

Exercises the real framework dispute lifecycle via the Rust CLI in local mode
(mirroring run_chain_onboarding.py), then builds and validates dispute_record,
dispute_resolution, and settlement_receipt protocol envelopes.
"""
import base64
import hashlib
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


ROOT = Path(__file__).resolve().parents[2]
FRAMEWORK_DIR = ROOT / "framework"
DEMO_DIR = ROOT / "demo"
FIXTURES_DIR = DEMO_DIR / "fixtures"
ARTIFACTS_DIR = DEMO_DIR / "artifacts"
SCHEMA_PATH = ROOT / "spec" / "schemas" / "protocol-envelope.v1.schema.json"

TYPE_TO_DEF = {
    "objective": "objectivePayload",
    "authority_grant": "authorityGrantPayload",
    "offer": "offerPayload",
    "acceptance": "acceptancePayload",
    "evidence_record": "evidenceRecordPayload",
    "settlement_receipt": "settlementReceiptPayload",
    "dispute_record": "disputeRecordPayload",
    "dispute_resolution": "disputeResolutionPayload",
}


def run_cli(args: List[str]) -> Dict[str, Any]:
    cmd = ["cargo", "run", "--quiet", "--"] + args
    result = subprocess.run(
        cmd, cwd=FRAMEWORK_DIR, text=True, capture_output=True, check=False
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


def canonical_json_bytes(payload: Dict[str, Any]) -> bytes:
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def digest(payload: Dict[str, Any]) -> str:
    return f"sha256:{hashlib.sha256(canonical_json_bytes(payload)).hexdigest()}"


def now_rfc3339() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def build_envelope(
    object_type: str, object_id: str, issuer_id: str, payload: Dict[str, Any], kid: str
) -> Dict[str, Any]:
    payload_hash = hashlib.sha256(canonical_json_bytes(payload)).hexdigest()
    signature = base64.urlsafe_b64encode(
        hashlib.sha256((object_id + kid).encode("utf-8")).digest()
    ).decode("utf-8")
    return {
        "object_type": object_type,
        "object_version": "1.0",
        "object_id": object_id,
        "issued_at": now_rfc3339(),
        "issuer_id": issuer_id,
        "payload": payload,
        "integrity": {
            "canonicalization": "jcs-rfc8785",
            "hash_alg": "sha256",
            "hash": f"sha256:{payload_hash}",
            "signature": {"alg": "Ed25519", "kid": kid, "value": signature},
        },
    }


def validate_envelope(envelope: Dict[str, Any], schema: Dict[str, Any]) -> None:
    for key in schema.get("required", []):
        if key not in envelope:
            raise ValueError(f"missing envelope key: {key}")
    object_type = envelope["object_type"]
    if object_type not in schema["properties"]["object_type"]["enum"]:
        raise ValueError(f"invalid object_type: {object_type}")
    payload_required = schema["$defs"][TYPE_TO_DEF[object_type]].get("required", [])
    for key in payload_required:
        if key not in envelope["payload"]:
            raise ValueError(f"missing payload key for {object_type}: {key}")
    payload_hash = hashlib.sha256(canonical_json_bytes(envelope["payload"])).hexdigest()
    if envelope["integrity"]["hash"] != f"sha256:{payload_hash}":
        raise ValueError("integrity hash mismatch")


def main() -> int:
    ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
    scenario = json.loads((FIXTURES_DIR / "dispute.demo.json").read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))

    commitment = scenario["commitment"]
    accepted_offer = scenario["accepted_offer"]
    provider_receipt = scenario["provider_receipt"]
    dispute_spec = scenario["dispute"]
    expected = scenario["expected_resolution"]

    # Fresh local framework state for a self-contained, offline run.
    run_cli(["init"])

    # 1) Seed the accepted commitment (business-class offer from Charlie).
    commitment_out = run_cli(
        [
            "commitment-record",
            "--commitment-id", commitment["commitment_ref"],
            "--objective-ref", commitment["objective_ref"],
            "--offer-id", commitment["offer_id"],
            "--accepted-by", commitment["accepted_by"],
            "--provider-id", commitment["provider_id"],
            "--assigned-verifier", commitment["assigned_verifier"],
            "--settlement-authority", commitment["settlement_authority"],
            "--settlement-state", commitment["settlement_state"],
        ]
    )

    # 2) Evidence: accepted offer vs provider fulfillment receipt (mismatch).
    accepted_offer_digest = digest(accepted_offer)
    provider_receipt_digest = digest(provider_receipt)
    dispute_evidence = {
        "statement": dispute_spec["statement"],
        "evidence_refs": [
            {"type": "accepted_offer", "digest": accepted_offer_digest},
            {"type": "provider_receipt", "digest": provider_receipt_digest},
        ],
    }
    evidence_path = ARTIFACTS_DIR / "dispute-evidence.generated.json"
    evidence_path.write_text(json.dumps(dispute_evidence, indent=2), encoding="utf-8")

    events: List[Dict[str, Any]] = []
    settlement_trail: List[str] = [commitment["settlement_state"]]
    events.append(
        {
            "event": "commitment_recorded",
            "commitment_ref": commitment["commitment_ref"],
            "settlement_state": commitment["settlement_state"],
        }
    )
    events.append(
        {
            "event": "evidence_recorded",
            "provider_id": commitment["provider_id"],
            "evidence_type": provider_receipt["evidence_type"],
            "digest": provider_receipt_digest,
        }
    )

    # 3) Agent opens a terms_mismatch dispute.
    dispute = run_cli(
        [
            "dispute-open",
            "--commitment-id", commitment["commitment_ref"],
            "--reason", dispute_spec["reason_code"],
            "--evidence", str(evidence_path),
            "--raised-by", dispute_spec["raised_by"],
            "--response-deadline", dispute_spec["response_deadline"],
        ]
    )
    dispute_id = dispute["dispute_id"]
    settlement_trail.append("disputed")
    events.append(
        {
            "event": "dispute_opened",
            "dispute_id": dispute_id,
            "reason_code": dispute["reason_code"],
            "raised_by": dispute["raised_by"],
            "settlement_state": "disputed",
        }
    )

    # 4) Verifier compares accepted terms vs delivered terms.
    accepted_cabin = accepted_offer["terms"]["cabin"]
    delivered_cabin = provider_receipt["delivered_terms"]["cabin"]
    mismatch = accepted_cabin != delivered_cabin
    verifier_review = {
        "verifier": commitment["assigned_verifier"],
        "accepted_cabin": accepted_cabin,
        "delivered_cabin": delivered_cabin,
        "mismatch_detected": mismatch,
        "recommended_decision": expected["decision"] if mismatch else "fulfilled",
    }
    verifier_report_digest = digest(verifier_review)
    events.append(
        {
            "event": "verifier_review_complete",
            "verifier": commitment["assigned_verifier"],
            "mismatch_detected": mismatch,
            "recommended_decision": verifier_review["recommended_decision"],
        }
    )

    # 5) Resolve: refunded, recorded by the assigned verifier.
    resolution_input = {
        "resolution_evidence_refs": [
            {"type": "verifier_report", "digest": verifier_report_digest}
        ]
    }
    resolution_path = ARTIFACTS_DIR / "dispute-resolution.generated.json"
    resolution_path.write_text(json.dumps(resolution_input, indent=2), encoding="utf-8")

    resolved = run_cli(
        [
            "dispute-resolve",
            "--dispute-id", dispute_id,
            "--decision", expected["decision"],
            "--resolution", str(resolution_path),
            "--resolved-by", expected["resolved_by"],
        ]
    )
    resolution = resolved["resolution"]
    settlement_trail.append(resolution["settlement_state"])
    events.append(
        {
            "event": "dispute_resolved",
            "dispute_id": dispute_id,
            "decision": resolution["decision"],
            "resolved_by": resolution["resolved_by"],
            "settlement_state": resolution["settlement_state"],
        }
    )
    events.append(
        {
            "event": "settlement_recorded",
            "commitment_ref": commitment["commitment_ref"],
            "state": resolution["settlement_state"],
        }
    )

    # Build + validate protocol envelopes.
    envelopes: List[Dict[str, Any]] = []

    dispute_payload = {
        "dispute_id": dispute_id,
        "commitment_ref": commitment["commitment_ref"],
        "raised_by": dispute["raised_by"],
        "reason_code": dispute["reason_code"],
        "statement": dispute.get("statement"),
        "evidence_refs": dispute["evidence_refs"],
        "opened_at": dispute["opened_at"],
        "response_deadline": dispute["response_deadline"],
        "status": "resolved",
    }
    dispute_env = build_envelope(
        "dispute_record",
        f"env_{dispute_id}",
        dispute["raised_by"],
        dispute_payload,
        "agent-demo-ed25519-2026-q1",
    )
    validate_envelope(dispute_env, schema)
    envelopes.append(dispute_env)

    resolution_payload = {
        "resolution_id": resolution["resolution_id"],
        "dispute_id": dispute_id,
        "commitment_ref": commitment["commitment_ref"],
        "decision": resolution["decision"],
        "resolved_by": resolution["resolved_by"],
        "resolution_evidence_refs": resolution["resolution_evidence_refs"],
        "resolved_at": resolution["resolved_at"],
        "settlement_state": resolution["settlement_state"],
    }
    resolution_env = build_envelope(
        "dispute_resolution",
        f"env_{resolution['resolution_id']}",
        commitment["assigned_verifier"],
        resolution_payload,
        "verifier-demo-ed25519-2026-q1",
    )
    validate_envelope(resolution_env, schema)
    envelopes.append(resolution_env)

    settlement_payload = {
        "receipt_id": "set_demo_dispute_001",
        "commitment_ref": commitment["commitment_ref"],
        "state": resolution["settlement_state"],
        "recorded_at": now_rfc3339(),
        "evidence_refs": [resolution["resolution_id"]],
    }
    settlement_env = build_envelope(
        "settlement_receipt",
        "env_settlement_dispute_demo_001",
        commitment["settlement_authority"],
        settlement_payload,
        "settlement-demo-ed25519-2026-q1",
    )
    validate_envelope(settlement_env, schema)
    envelopes.append(settlement_env)

    result = {
        "scenario": scenario["scenario"],
        "commitment": commitment_out,
        "dispute": dispute,
        "verifier_review": verifier_review,
        "resolution": resolution,
        "settlement_state_trail": settlement_trail,
        "events": events,
        "envelopes": envelopes,
    }
    (ARTIFACTS_DIR / "dispute-flow.demo.json").write_text(
        json.dumps(result, indent=2), encoding="utf-8"
    )

    print(
        json.dumps(
            {
                "ok": True,
                "dispute_id": dispute_id,
                "reason_code": dispute["reason_code"],
                "mismatch_detected": mismatch,
                "decision": resolution["decision"],
                "settlement_state_trail": settlement_trail,
                "artifact": str(ARTIFACTS_DIR / "dispute-flow.demo.json"),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:  # noqa: BLE001 - demo entrypoint surfaces errors as JSON
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2))
        raise
