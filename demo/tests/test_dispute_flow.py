"""End-to-end test for the offline terms-mismatch dispute + refund flow.

This runs the real demo pipeline (`run_dispute_flow.py`), which exercises the
Rust framework dispute lifecycle via the CLI in local mode, then asserts the
produced artifact, settlement-state trail, events, and protocol envelopes. It is
skipped automatically when the Rust toolchain (`cargo`) is unavailable.
"""
import json
import shutil
import subprocess
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "demo" / "scripts"
ARTIFACT = ROOT / "demo" / "artifacts" / "dispute-flow.demo.json"
ENVELOPE_SCHEMA = json.loads(
    (ROOT / "spec" / "schemas" / "protocol-envelope.v1.schema.json").read_text(encoding="utf-8")
)


@unittest.skipUnless(shutil.which("cargo"), "cargo (Rust toolchain) not available")
class DisputeFlowIntegrationTest(unittest.TestCase):
    data = None

    @classmethod
    def setUpClass(cls):
        result = subprocess.run(
            ["python3", str(SCRIPTS / "run_dispute_flow.py")],
            cwd=str(ROOT),
            text=True,
            capture_output=True,
            check=False,
        )
        if result.returncode != 0:
            raise AssertionError(
                f"run_dispute_flow.py failed:\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}"
            )
        cls.data = json.loads(ARTIFACT.read_text(encoding="utf-8"))

    def test_scenario_is_terms_mismatch_refund(self):
        self.assertEqual(self.data["scenario"], "terms_mismatch_refund")

    def test_dispute_opened_with_terms_mismatch(self):
        dispute = self.data["dispute"]
        self.assertEqual(dispute["reason_code"], "terms_mismatch")
        self.assertEqual(dispute["raised_by"], "agent.demo.orchestrator")

    def test_verifier_detects_cabin_mismatch(self):
        review = self.data["verifier_review"]
        self.assertTrue(review["mismatch_detected"])
        self.assertEqual(review["accepted_cabin"], "business")
        self.assertEqual(review["delivered_cabin"], "premium_economy")

    def test_resolution_is_refunded(self):
        resolution = self.data["resolution"]
        self.assertEqual(resolution["decision"], "refunded")
        self.assertEqual(resolution["settlement_state"], "refunded")
        self.assertEqual(resolution["resolved_by"], "verifier.demo.audit")

    def test_settlement_state_trail(self):
        self.assertEqual(
            self.data["settlement_state_trail"], ["fulfilled", "disputed", "refunded"]
        )

    def test_expected_events_present(self):
        names = {e["event"] for e in self.data["events"]}
        for expected in [
            "commitment_recorded",
            "evidence_recorded",
            "dispute_opened",
            "verifier_review_complete",
            "dispute_resolved",
            "settlement_recorded",
        ]:
            self.assertIn(expected, names)

    def test_envelopes_present_and_valid(self):
        validator = Draft202012Validator(ENVELOPE_SCHEMA)
        object_types = set()
        for env in self.data["envelopes"]:
            errors = list(validator.iter_errors(env))
            self.assertEqual(
                errors, [], f"envelope {env.get('object_id')} failed schema: {errors}"
            )
            object_types.add(env["object_type"])
        for expected in ["dispute_record", "dispute_resolution", "settlement_receipt"]:
            self.assertIn(expected, object_types)

    def test_evidence_digests_are_sha256(self):
        for ref in self.data["dispute"]["evidence_refs"]:
            self.assertRegex(ref["digest"], r"^sha256:[a-f0-9]{64}$")


if __name__ == "__main__":
    unittest.main()
