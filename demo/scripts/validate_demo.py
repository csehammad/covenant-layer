#!/usr/bin/env python3
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ARTIFACT = ROOT / "demo" / "artifacts" / "demo.full.json"


def fail(msg: str) -> None:
    raise SystemExit(f"VALIDATION_FAILED: {msg}")


def main() -> int:
    if not ARTIFACT.exists():
        fail(f"missing artifact: {ARTIFACT}")
    data = json.loads(ARTIFACT.read_text(encoding="utf-8"))
    onboarding = data.get("onboarding", {})
    objective_flow = data.get("objective_flow", {})

    participants = onboarding.get("participants", [])
    if len(participants) < 4:
        fail("expected at least 4 participants (agent + 3 providers)")

    required_ids = {
        "agent.demo.orchestrator",
        "provider.demo.alpha",
        "provider.demo.bravo",
        "provider.demo.charlie",
    }
    present_ids = {p.get("participant_id") for p in participants}
    if not required_ids.issubset(present_ids):
        fail(f"missing participants: {sorted(required_ids - present_ids)}")

    for p in participants:
        final_state = None
        for step in p.get("steps", []):
            output = step.get("output", {})
            if isinstance(output, dict) and "state" in output:
                final_state = output["state"]
        if final_state != "active":
            fail(f"participant {p.get('participant_id')} final state is not active")

    scored_offers = objective_flow.get("scored_offers", [])
    if len(scored_offers) != 3:
        fail("expected 3 scored offers")
    if not objective_flow.get("selected_offer"):
        fail("missing selected_offer")

    events = objective_flow.get("events", [])
    event_names = {e.get("event") for e in events}
    for expected in ["objective_published", "offer_accepted", "evidence_recorded", "settlement_recorded"]:
        if expected not in event_names:
            fail(f"missing event: {expected}")

    print("VALIDATION_OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
