# Wire Format, Integrity, and Errors

This section defines implementation requirements needed for interoperable protocol exchange.

Reference JSON Schema:

- `schemas/protocol-envelope.v1.schema.json`

## Normative language

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## Transport envelope

Protocol objects may be transported over HTTP, queues, or other channels, but each exchanged object MUST include:

- `object_type`
- `object_version`
- `object_id`
- `issued_at` (RFC3339 UTC)
- `issuer_id`
- `payload`
- `integrity`

Recommended envelope shape:

```yaml
object_type: offer
object_version: "1.0"
object_id: "ofr_01H..."
issued_at: "2026-03-12T12:00:00Z"
issuer_id: "provider.example.travel"
payload: {}
integrity:
  canonicalization: "jcs-rfc8785"
  hash_alg: "sha256"
  hash: "sha256:..."
  signature:
    alg: "Ed25519"
    kid: "provider-ed25519-2026-q1"
    value: "base64url..."
```

## Canonicalization and hashing

To prevent signature ambiguity:

- JSON payloads MUST use deterministic JSON canonicalization (`jcs-rfc8785`) before hashing/signing.
- YAML payloads MUST be converted to deterministic JSON first, then canonicalized with JCS.
- Hash values MUST be generated over canonicalized payload bytes.
- `sha256` is REQUIRED for baseline interoperability.

Signatures that cannot be recomputed from canonical payload bytes MUST be rejected.

## Signature and key binding

- `integrity.signature.kid` MUST resolve to a currently valid signing key for `issuer_id`.
- Key validity windows (`valid_from`, `valid_to`) MUST be enforced.
- Expired keys MUST NOT be accepted for new object issuance.
- Unknown `kid` values MUST cause validation failure.

## Object version policy

- Every object MUST include `object_version`.
- Backward-compatible field additions SHOULD increment minor version.
- Breaking semantic changes MUST increment major version.
- Receivers MUST fail closed on unknown mandatory semantics.

## Required object fields by type

Minimum payload fields:

- **Objective:** `objective_id`, `principal_id`, `target_outcome`, `constraints`, `expiry`
- **Authority Grant:** `grant_id`, `issuer`, `holder`, `allowed_actions`, `expiry`, `revocation_ref`
- **Offer:** `offer_id`, `provider_id`, `objective_id`, `terms`, `expires_at`, `acceptance_conditions`
- **Acceptance:** `acceptance_id`, `offer_id`, `accepted_by`, `accepted_at`, `approval_ref` (if required)
- **Evidence Record:** `evidence_id`, `commitment_ref`, `submitted_by`, `evidence_type`, `evidence_ref`, `submitted_at`
- **Settlement Receipt:** `receipt_id`, `commitment_ref`, `state`, `recorded_at`, `evidence_refs`

Objects missing required fields MUST be invalid.

## Idempotency and replay protection

- Mutating operations MUST include an idempotency key.
- Reuse of the same idempotency key with different semantic payload MUST be rejected.
- Objects with expired validity windows MUST NOT advance lifecycle state.
- Duplicate object identifiers MUST be treated as replay or conflict and handled explicitly.

## Error taxonomy

Implementations SHOULD expose stable machine-readable errors.

Baseline error codes:

- `ERR_INVALID_SCHEMA`
- `ERR_INVALID_SIGNATURE`
- `ERR_UNKNOWN_SIGNING_KEY`
- `ERR_EXPIRED_OBJECT`
- `ERR_REVOKED_AUTHORITY`
- `ERR_INVALID_STATE_TRANSITION`
- `ERR_PROFILE_UNSUPPORTED`
- `ERR_ELIGIBILITY_BLOCKED`
- `ERR_REPLAY_DETECTED`
- `ERR_IDEMPOTENCY_CONFLICT`
- `ERR_EVIDENCE_INSUFFICIENT`
- `ERR_SETTLEMENT_CONFLICT`

Recommended error response shape:

```json
{
  "error_code": "ERR_INVALID_SIGNATURE",
  "message": "signature verification failed for offer ofr_01H...",
  "object_ref": "ofr_01H...",
  "retryable": false
}
```

## Auditability requirements

For every rejected or accepted lifecycle mutation, implementations SHOULD record:

- object reference
- decision result
- validator identity/service
- timestamp
- reason/error code

This enables deterministic post-incident reconstruction.
