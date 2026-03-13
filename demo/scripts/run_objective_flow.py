#!/usr/bin/env python3
import base64
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

sys.path.insert(0, str(Path(__file__).resolve().parent))
from agent_reasoning import reason_over_offers


ROOT = Path(__file__).resolve().parents[2]
DEMO_DIR = ROOT / "demo"
FIXTURES_DIR = DEMO_DIR / "fixtures"
ARTIFACTS_DIR = DEMO_DIR / "artifacts"
SCHEMA_PATH = ROOT / "spec" / "schemas" / "protocol-envelope.v1.schema.json"


def canonical_json_bytes(payload: Dict[str, Any]) -> bytes:
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def now_rfc3339() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def build_envelope(
    object_type: str,
    object_id: str,
    issuer_id: str,
    payload: Dict[str, Any],
    kid: str,
) -> Dict[str, Any]:
    payload_bytes = canonical_json_bytes(payload)
    digest = hashlib.sha256(payload_bytes).hexdigest()
    signature = base64.urlsafe_b64encode(hashlib.sha256((object_id + kid).encode("utf-8")).digest()).decode(
        "utf-8"
    )
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
            "hash": f"sha256:{digest}",
            "signature": {
                "alg": "Ed25519",
                "kid": kid,
                "value": signature,
            },
        },
    }


def validate_against_schema_baseline(envelope: Dict[str, Any], schema: Dict[str, Any]) -> None:
    required = schema.get("required", [])
    for key in required:
        if key not in envelope:
            raise ValueError(f"missing envelope key: {key}")

    object_type = envelope["object_type"]
    allowed_types = schema["properties"]["object_type"]["enum"]
    if object_type not in allowed_types:
        raise ValueError(f"invalid object_type: {object_type}")

    type_to_def = {
        "objective": "objectivePayload",
        "authority_grant": "authorityGrantPayload",
        "offer": "offerPayload",
        "acceptance": "acceptancePayload",
        "evidence_record": "evidenceRecordPayload",
        "settlement_receipt": "settlementReceiptPayload",
    }
    payload_required = schema["$defs"][type_to_def[object_type]].get("required", [])
    payload = envelope["payload"]
    for key in payload_required:
        if key not in payload:
            raise ValueError(f"missing payload key for {object_type}: {key}")

    payload_hash = hashlib.sha256(canonical_json_bytes(payload)).hexdigest()
    if envelope["integrity"]["hash"] != f"sha256:{payload_hash}":
        raise ValueError("integrity hash mismatch")


def score_offer(offer: Dict[str, Any], objective: Dict[str, Any]) -> Tuple[bool, float, List[str]]:
    terms = offer["terms"]
    constraints = objective["constraints"]
    reasons: List[str] = []

    if terms["price_usd"] > constraints["max_budget_usd"]:
        return False, -1.0, ["rejected: exceeds max budget"]
    if terms["stops"] > constraints["max_stops"]:
        return False, -1.0, ["rejected: exceeds max stops"]
    if constraints["no_overnight_layover"] and terms["overnight_layover"]:
        return False, -1.0, ["rejected: overnight layover not allowed"]

    price_norm = max(0.0, min(1.0, 1.0 - (terms["price_usd"] / constraints["max_budget_usd"])))
    convenience_norm = 1.0 if terms["stops"] == 0 else 0.7
    refund_norm = float(terms["refund_flexibility_score"])
    weights = objective["preferences"]
    score = (
        price_norm * float(weights["weight_price"])
        + convenience_norm * float(weights["weight_convenience"])
        + refund_norm * float(weights["weight_refund_flexibility"])
    )
    reasons.extend(
        [
            f"price component={price_norm:.3f}",
            f"convenience component={convenience_norm:.3f}",
            f"refund component={refund_norm:.3f}",
        ]
    )
    return True, score, reasons


def main() -> int:
    ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
    objective = json.loads((FIXTURES_DIR / "objective.demo.json").read_text(encoding="utf-8"))
    offers_doc = json.loads((FIXTURES_DIR / "offers.demo.json").read_text(encoding="utf-8"))
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))

    envelopes: List[Dict[str, Any]] = []
    events: List[Dict[str, Any]] = []

    objective_payload = {
        "objective_id": objective["objective_id"],
        "principal_id": objective["principal_id"],
        "target_outcome": objective["route"],
        "constraints": objective["constraints"],
        "expiry": objective["expiry"],
    }
    objective_env = build_envelope(
        "objective",
        "env_objective_demo_001",
        "agent.demo.orchestrator",
        objective_payload,
        "agent-demo-ed25519-2026-q1",
    )
    validate_against_schema_baseline(objective_env, schema)
    envelopes.append(objective_env)
    events.append({"event": "objective_published", "ref": objective_env["object_id"]})

    scored_offers: List[Dict[str, Any]] = []
    for raw_offer in offers_doc["offers"]:
        offer_payload = {
            "offer_id": raw_offer["offer_id"],
            "provider_id": raw_offer["provider_id"],
            "objective_id": raw_offer["objective_id"],
            "terms": raw_offer["terms"],
            "expires_at": raw_offer["expires_at"],
            "acceptance_conditions": raw_offer["acceptance_conditions"],
        }
        offer_env = build_envelope(
            "offer",
            f"env_{raw_offer['offer_id']}",
            raw_offer["provider_id"],
            offer_payload,
            f"{raw_offer['provider_id'].replace('.', '-')}-kid",
        )
        validate_against_schema_baseline(offer_env, schema)
        envelopes.append(offer_env)
        valid, score, reasons = score_offer(raw_offer, objective)
        scored_offers.append(
            {
                "offer_id": raw_offer["offer_id"],
                "provider_id": raw_offer["provider_id"],
                "valid": valid,
                "score": score,
                "reasons": reasons,
                "terms": raw_offer["terms"],
            }
        )
        events.append({"event": "offer_published", "ref": offer_env["object_id"], "provider_id": raw_offer["provider_id"]})

    valid_offers = [o for o in scored_offers if o["valid"]]
    valid_offers.sort(key=lambda x: x["score"], reverse=True)
    if not valid_offers:
        raise RuntimeError("No valid offers available for objective.")

    # --- LLM reasoning step (the part that makes the agent an agent) ---
    llm_result: Optional[Dict[str, Any]] = None
    try:
        llm_result = reason_over_offers(objective, offers_doc["offers"])
    except Exception as exc:
        print(f"[agent] LLM reasoning failed: {exc}. Falling back to deterministic scoring.")

    if llm_result and llm_result.get("selected_offer_id") and not llm_result.get("parse_error"):
        llm_pick = llm_result["selected_offer_id"]
        llm_match = [o for o in valid_offers if o["offer_id"] == llm_pick]
        if llm_match:
            selected = llm_match[0]
            selection_method = "llm_reasoning"
            print(f"[agent] LLM reasoning selected: {selected['offer_id']} (model: {llm_result.get('model')})")
            print(f"[agent] Confidence: {llm_result.get('confidence', 'unknown')}")
        else:
            selected = valid_offers[0]
            selection_method = "deterministic_fallback"
            print(f"[agent] LLM picked {llm_pick} but it was disqualified by constraints. Falling back to deterministic: {selected['offer_id']}")
    else:
        selected = valid_offers[0]
        selection_method = "deterministic"
        if llm_result is None:
            print(f"[agent] No LLM configured. Using deterministic scoring: {selected['offer_id']}")
        else:
            print(f"[agent] LLM response unusable. Using deterministic scoring: {selected['offer_id']}")

    events.append({
        "event": "agent_reasoning_complete",
        "selection_method": selection_method,
        "selected_offer_id": selected["offer_id"],
        "llm_model": llm_result.get("model") if llm_result else None,
    })

    acceptance_payload = {
        "acceptance_id": "acc_demo_001",
        "offer_id": selected["offer_id"],
        "accepted_by": "agent.demo.orchestrator",
        "accepted_at": now_rfc3339(),
        "approval_ref": objective["approval_policy"]["approval_reference"],
    }
    acceptance_env = build_envelope(
        "acceptance",
        "env_acceptance_demo_001",
        "agent.demo.orchestrator",
        acceptance_payload,
        "agent-demo-ed25519-2026-q1",
    )
    validate_against_schema_baseline(acceptance_env, schema)
    envelopes.append(acceptance_env)
    events.append({"event": "offer_accepted", "ref": acceptance_env["object_id"], "offer_id": selected["offer_id"]})

    evidence_payload = {
        "evidence_id": "evd_demo_001",
        "commitment_ref": acceptance_payload["acceptance_id"],
        "submitted_by": selected["provider_id"],
        "evidence_type": "booking_confirmation",
        "evidence_ref": "urn:demo:evidence:booking_confirmation:001",
        "submitted_at": now_rfc3339(),
    }
    evidence_env = build_envelope(
        "evidence_record",
        "env_evidence_demo_001",
        selected["provider_id"],
        evidence_payload,
        f"{selected['provider_id'].replace('.', '-')}-kid",
    )
    validate_against_schema_baseline(evidence_env, schema)
    envelopes.append(evidence_env)
    events.append({"event": "evidence_recorded", "ref": evidence_env["object_id"], "provider_id": selected["provider_id"]})

    settlement_payload = {
        "receipt_id": "set_demo_001",
        "commitment_ref": acceptance_payload["acceptance_id"],
        "state": "fulfilled",
        "recorded_at": now_rfc3339(),
        "evidence_refs": [evidence_payload["evidence_id"]],
    }
    settlement_env = build_envelope(
        "settlement_receipt",
        "env_settlement_demo_001",
        "settlement.demo.service",
        settlement_payload,
        "settlement-demo-ed25519-2026-q1",
    )
    validate_against_schema_baseline(settlement_env, schema)
    envelopes.append(settlement_env)
    events.append({"event": "settlement_recorded", "ref": settlement_env["object_id"], "state": "fulfilled"})

    agent_reasoning_output = None
    if llm_result:
        agent_reasoning_output = {
            "model": llm_result.get("model"),
            "confidence": llm_result.get("confidence"),
            "reasoning": llm_result.get("reasoning"),
            "ranked_offers": llm_result.get("ranked_offers", []),
            "selected_offer_id": llm_result.get("selected_offer_id"),
        }

    result = {
        "objective_id": objective["objective_id"],
        "scored_offers": scored_offers,
        "agent_reasoning": agent_reasoning_output,
        "selection_method": selection_method,
        "selected_offer": selected,
        "events": events,
    }
    (ARTIFACTS_DIR / "objective-flow.demo.json").write_text(
        json.dumps(result, indent=2),
        encoding="utf-8",
    )
    (ARTIFACTS_DIR / "protocol-envelopes.demo.json").write_text(
        json.dumps({"envelopes": envelopes}, indent=2),
        encoding="utf-8",
    )
    print(
        json.dumps(
            {
                "ok": True,
                "selection_method": selection_method,
                "selected_offer": selected["offer_id"],
                "llm_model": llm_result.get("model") if llm_result else None,
                "llm_confidence": llm_result.get("confidence") if llm_result else None,
                "artifacts": {
                    "objective_flow": str(ARTIFACTS_DIR / "objective-flow.demo.json"),
                    "envelopes": str(ARTIFACTS_DIR / "protocol-envelopes.demo.json"),
                },
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
