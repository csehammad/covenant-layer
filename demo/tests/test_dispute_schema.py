"""Schema-level tests for the versioned dispute record and resolution payloads.

These tests validate crafted objects against the real JSON Schemas shipped in
`spec/schemas/` using jsonschema's Draft 2020-12 validator, so they exercise the
enum, pattern, and required-field constraints exactly as a conforming receiver
would. They require no Rust toolchain.
"""
import json
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[2]
ENVELOPE_SCHEMA = json.loads(
    (ROOT / "spec" / "schemas" / "protocol-envelope.v1.schema.json").read_text(encoding="utf-8")
)
DISPUTE_SCHEMA = json.loads(
    (ROOT / "spec" / "schemas" / "dispute-record.v1.schema.json").read_text(encoding="utf-8")
)

SHA = "sha256:" + "a" * 64
SHA2 = "sha256:" + "b" * 64


def valid_dispute_record():
    return {
        "dispute_id": "dsp_demo_charlie_001",
        "commitment_ref": "cmt_demo_charlie_001",
        "raised_by": "agent.demo.orchestrator",
        "reason_code": "terms_mismatch",
        "statement": "Booked cabin does not match the accepted offer",
        "evidence_refs": [
            {"type": "accepted_offer", "digest": SHA},
            {"type": "provider_receipt", "digest": SHA2},
        ],
        "opened_at": "2026-05-21T12:00:00Z",
        "response_deadline": "2026-05-24T12:00:00Z",
        "status": "open",
    }


def valid_dispute_resolution():
    return {
        "resolution_id": "res_demo_charlie_001",
        "dispute_id": "dsp_demo_charlie_001",
        "commitment_ref": "cmt_demo_charlie_001",
        "decision": "refunded",
        "resolved_by": "verifier.demo.audit",
        "resolution_evidence_refs": [{"type": "verifier_report", "digest": SHA}],
        "resolved_at": "2026-05-23T12:00:00Z",
        "settlement_state": "refunded",
    }


def payload_validator(def_name):
    """Validator for a single payload $def from the standalone dispute schema."""
    schema = {"$ref": f"#/$defs/{def_name}", "$defs": DISPUTE_SCHEMA["$defs"]}
    return Draft202012Validator(schema)


def is_valid(validator, instance):
    return not list(validator.iter_errors(instance))


class DisputeRecordSchemaTest(unittest.TestCase):
    def setUp(self):
        self.validator = payload_validator("disputeRecordPayload")

    def test_valid_record_passes(self):
        self.assertTrue(is_valid(self.validator, valid_dispute_record()))

    def test_statement_is_optional(self):
        record = valid_dispute_record()
        record.pop("statement")
        self.assertTrue(is_valid(self.validator, record))

    def test_unknown_reason_code_rejected(self):
        record = valid_dispute_record()
        record["reason_code"] = "made_up_reason"
        self.assertFalse(is_valid(self.validator, record))

    def test_all_baseline_reason_codes_accepted(self):
        for code in [
            "terms_mismatch",
            "missing_evidence",
            "late_fulfillment",
            "duplicate_charge",
            "non_delivery",
        ]:
            record = valid_dispute_record()
            record["reason_code"] = code
            self.assertTrue(is_valid(self.validator, record), f"reason_code {code} should pass")

    def test_invalid_status_rejected(self):
        record = valid_dispute_record()
        record["status"] = "in_progress"
        self.assertFalse(is_valid(self.validator, record))

    def test_missing_required_field_rejected(self):
        for field in [
            "dispute_id",
            "commitment_ref",
            "raised_by",
            "reason_code",
            "evidence_refs",
            "opened_at",
            "response_deadline",
            "status",
        ]:
            record = valid_dispute_record()
            record.pop(field)
            self.assertFalse(
                is_valid(self.validator, record), f"missing {field} should be rejected"
            )

    def test_empty_evidence_refs_rejected(self):
        record = valid_dispute_record()
        record["evidence_refs"] = []
        self.assertFalse(is_valid(self.validator, record))

    def test_bad_digest_pattern_rejected(self):
        record = valid_dispute_record()
        record["evidence_refs"][0]["digest"] = "md5:deadbeef"
        self.assertFalse(is_valid(self.validator, record))

    def test_evidence_ref_requires_type_and_digest(self):
        record = valid_dispute_record()
        record["evidence_refs"] = [{"type": "accepted_offer"}]
        self.assertFalse(is_valid(self.validator, record))


class DisputeResolutionSchemaTest(unittest.TestCase):
    def setUp(self):
        self.validator = payload_validator("disputeResolutionPayload")

    def test_valid_resolution_passes(self):
        self.assertTrue(is_valid(self.validator, valid_dispute_resolution()))

    def test_invalid_decision_rejected(self):
        resolution = valid_dispute_resolution()
        resolution["decision"] = "disputed"
        self.assertFalse(is_valid(self.validator, resolution))

    def test_all_terminal_decisions_accepted(self):
        for decision in ["fulfilled", "failed", "refunded"]:
            resolution = valid_dispute_resolution()
            resolution["decision"] = decision
            resolution["settlement_state"] = decision
            self.assertTrue(is_valid(self.validator, resolution))

    def test_missing_resolved_by_rejected(self):
        resolution = valid_dispute_resolution()
        resolution.pop("resolved_by")
        self.assertFalse(is_valid(self.validator, resolution))


class EnvelopeBindingTest(unittest.TestCase):
    """The transport envelope must bind dispute payloads by object_type."""

    def setUp(self):
        self.validator = Draft202012Validator(ENVELOPE_SCHEMA)

    def _envelope(self, object_type, payload):
        return {
            "object_type": object_type,
            "object_version": "1.0",
            "object_id": "env_test_001",
            "issued_at": "2026-05-21T12:00:00Z",
            "issuer_id": "agent.demo.orchestrator",
            "payload": payload,
            "integrity": {
                "canonicalization": "jcs-rfc8785",
                "hash_alg": "sha256",
                "hash": "sha256:" + "c" * 64,
                "signature": {
                    "alg": "Ed25519",
                    "kid": "agent-demo-ed25519-2026-q1",
                    "value": "dGVzdC1zaWduYXR1cmUtdmFsdWU",
                },
            },
        }

    def test_dispute_record_object_type_enum_present(self):
        enum = ENVELOPE_SCHEMA["properties"]["object_type"]["enum"]
        self.assertIn("dispute_record", enum)
        self.assertIn("dispute_resolution", enum)

    def test_valid_dispute_record_envelope_passes(self):
        env = self._envelope("dispute_record", valid_dispute_record())
        self.assertTrue(is_valid(self.validator, env))

    def test_valid_dispute_resolution_envelope_passes(self):
        env = self._envelope("dispute_resolution", valid_dispute_resolution())
        self.assertTrue(is_valid(self.validator, env))

    def test_envelope_binds_payload_by_type(self):
        # A resolution payload carried under object_type dispute_record must fail.
        env = self._envelope("dispute_record", valid_dispute_resolution())
        self.assertFalse(is_valid(self.validator, env))

    def test_bad_reason_code_fails_through_envelope(self):
        payload = valid_dispute_record()
        payload["reason_code"] = "not_a_reason"
        env = self._envelope("dispute_record", payload)
        self.assertFalse(is_valid(self.validator, env))


if __name__ == "__main__":
    unittest.main()
