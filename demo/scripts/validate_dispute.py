#!/usr/bin/env python3
"""Validate the offline dispute + refund artifact. Prints DISPUTE_VALIDATION_OK."""
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ARTIFACT = ROOT / "demo" / "artifacts" / "dispute-flow.demo.json"


def fail(msg: str) -> None:
    raise SystemExit(f"DISPUTE_VALIDATION_FAILED: {msg}")


def main() -> int:
    if not ARTIFACT.exists():
        fail(f"missing artifact: {ARTIFACT}")
    data = json.loads(ARTIFACT.read_text(encoding="utf-8"))

    dispute = data.get("dispute", {})
    if dispute.get("reason_code") != "terms_mismatch":
        fail("dispute reason_code must be terms_mismatch")

    resolution = data.get("resolution", {})
    if resolution.get("decision") != "refunded":
        fail("resolution decision must be refunded")
    if resolution.get("settlement_state") != "refunded":
        fail("resolution settlement_state must be refunded")

    review = data.get("verifier_review", {})
    if not review.get("mismatch_detected"):
        fail("verifier review must detect a mismatch")

    trail = data.get("settlement_state_trail", [])
    if "disputed" not in trail:
        fail("settlement trail must include disputed")
    if trail[-1] != "refunded":
        fail("settlement trail must end in refunded")

    event_names = {e.get("event") for e in data.get("events", [])}
    for expected in ["dispute_opened", "verifier_review_complete", "dispute_resolved", "settlement_recorded"]:
        if expected not in event_names:
            fail(f"missing event: {expected}")

    object_types = {e.get("object_type") for e in data.get("envelopes", [])}
    for expected in ["dispute_record", "dispute_resolution", "settlement_receipt"]:
        if expected not in object_types:
            fail(f"missing envelope: {expected}")

    print("DISPUTE_VALIDATION_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
