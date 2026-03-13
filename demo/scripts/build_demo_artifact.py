#!/usr/bin/env python3
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ARTIFACTS = ROOT / "demo" / "artifacts"


def main() -> int:
    onboarding_file = ARTIFACTS / "onboarding.demo.json"
    flow_file = ARTIFACTS / "objective-flow.demo.json"

    onboarding = json.loads(onboarding_file.read_text(encoding="utf-8")) if onboarding_file.exists() else {}
    flow = json.loads(flow_file.read_text(encoding="utf-8")) if flow_file.exists() else {}

    full = {
        "demo_name": "chain_only_e2e_demo",
        "onboarding": onboarding,
        "objective_flow": flow,
    }
    out = ARTIFACTS / "demo.full.json"
    out.write_text(json.dumps(full, indent=2), encoding="utf-8")
    print(json.dumps({"ok": True, "artifact": str(out)}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
