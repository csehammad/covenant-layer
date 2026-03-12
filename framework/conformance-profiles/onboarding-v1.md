# Conformance Profile: onboarding-v1

This document defines the normative conformance requirements for participant manifests that declare:

```yaml
conformance_profile: onboarding-v1
```

## Scope

`onboarding-v1` is the baseline profile for testnet onboarding and activation gating across:

- `provider`
- `broker`
- `settlement_service`
- `verifier`
- `agent_service`

## Global requirements (all participant types)

Every manifest MUST:

- be valid YAML and pass schema-level required fields
- declare `conformance_profile: onboarding-v1`
- include identity metadata and at least one signing key
- include declared service interfaces
- initialize `admission_status.state` as `requested`
- use detached signatures (`signature.mode: detached`)

## Actor-specific requirements

### Provider

Manifest MUST include:

- non-empty `capabilities.objective_types`
- non-empty `evidence_profile.required_types`
- `provider_policy.incident_response_policy_ref` (non-empty string)
- `provider_policy.data_protection_policy_ref` (non-empty string)
- `provider_policy.fulfillment_sla_seconds` (positive integer)
- `provider_policy.idempotency_window_seconds` (positive integer)

### Broker

Manifest MUST include:

- non-empty `broker_policy.routing_scope`
- `broker_policy.offer_provenance_policy_ref` (non-empty string)
- `broker_policy.conflict_of_interest_policy_ref` (non-empty string)
- `broker_policy.routing_audit_log_retention_days` (positive integer)
- `broker_policy.fairness_review_interval_days` (positive integer)

### Settlement Service

Manifest MUST include:

- non-empty `settlement_policy.supported_states`
- `settlement_policy.dispute_transition_policy_ref` (non-empty string)
- `settlement_policy.evidence_link_integrity_ref` (non-empty string)
- `settlement_policy.finality_wait_seconds` (positive integer)
- `settlement_policy.replay_protection: true`

### Verifier

Manifest MUST include:

- non-empty `verification_policy.verification_domains`
- `verification_policy.replay_protection: true`
- `verification_policy.verification_sla_seconds` (positive integer)
- `verification_policy.decision_policy_ref` (non-empty string)

### Agent Service

Manifest MUST include:

- `agent_policy.authority_handling_policy_ref` (non-empty string)
- `agent_policy.approval_gating_required: true`
- `agent_policy.authority_proof_method` (non-empty string)
- `agent_policy.approval_reference_format` (non-empty string)
- `agent_policy.grant_revocation_check_seconds` (positive integer)
- `agent_policy.max_authority_ttl_seconds` (positive integer)

## Activation policy

Passing conformance checks is necessary but not sufficient for activation. Promotion from `probation` to `active` SHOULD require governance attestations according to the actor runbooks.

## Versioning and migration

- Profile identifiers are immutable once published.
- Breaking changes require a new profile version (for example, `onboarding-v2`).
- Governance SHOULD publish migration timelines and compatibility notes before changing required profile versions.
