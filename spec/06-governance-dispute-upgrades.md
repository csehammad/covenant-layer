# Governance, Dispute, and Upgrade Policy

This section defines minimum governance and operational policy needed to keep the protocol decentralized, auditable, and evolvable.

## Normative language

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are normative.

## Governance responsibilities

A participating network MUST define governance authority for:

- conformance profile policy
- participant state transition authority
- emergency restriction and revocation actions
- protocol/profile upgrade approvals
- dispute escalation and closure policy

Governance authority MAY be DAO, multisig, federation, or equivalent, but it MUST be explicit and publicly documented.

## Change control and upgrades

### Required controls

- Breaking protocol changes MUST use explicit version increments.
- Breaking conformance changes MUST use new profile identifiers (for example, `onboarding-v2`).
- Upgrade proposals MUST include rationale, compatibility impact, and migration timeline.
- Enforced version changes MUST provide a deprecation window unless emergency security risk exists.

### Emergency path

Emergency actions (for example, emergency restriction of compromised participants):

- MUST be narrowly scoped
- MUST be time-bounded
- MUST produce an auditable event trail
- SHOULD require post-incident public review

## Participant state authority

State transitions MUST be policy-bound and traceable.

At minimum:

- only authorized governance actors can perform non-self transitions
- each transition MUST include reason metadata
- transition events MUST be queryable
- transition actions SHOULD include attestation references when relevant

## Dispute lifecycle

### Entry criteria

A commitment MAY enter `disputed` when:

- fulfillment terms appear violated
- evidence is conflicting or insufficient
- timing/expiry terms were not met
- authorization validity is contested

### Required dispute record

A dispute record MUST include:

- `dispute_id`
- `commitment_ref`
- `raised_by`
- `reason_code`
- `opened_at`
- `response_deadline`
- `evidence_refs`
- `status`

A dispute record MAY include:

- `statement` — supplementary free-text context

The `statement` field is supplementary only. Settlement logic MUST NOT depend
on interpreting prose; machine decisions MUST be driven by `reason_code`,
`evidence_refs`, and recorded settlement state.

The canonical wire schema for dispute records and resolutions is
`schemas/dispute-record.v1.schema.json`, transported using the standard
protocol envelope (`object_type: dispute_record` and
`object_type: dispute_resolution`).

### Reason codes

`reason_code` MUST be drawn from a fixed, network-published set. The baseline
set is:

- `terms_mismatch` — delivered outcome does not match the accepted offer
- `missing_evidence` — required fulfillment evidence was not submitted
- `late_fulfillment` — fulfillment occurred after the valid window
- `duplicate_charge` — the commitment was charged/settled more than once
- `non_delivery` — no fulfillment occurred

### Dispute statuses

Minimum statuses:

- `open`
- `investigating`
- `resolved`
- `closed`

### Evidence immutability

- `evidence_refs` attached to a dispute MUST be treated as immutable after the
  `response_deadline` has passed.
- Attempts to append or alter dispute evidence after the deadline MUST be
  rejected.

### Authorization

- Only a party to the accepted commitment MAY open a dispute for that
  commitment.
- Only the assigned verifier or the settlement authority MAY resolve a dispute.
- A resolved dispute MUST NOT be reopened under the same `dispute_id`.
- A duplicate `dispute_id` MUST be treated as replay and rejected.

### Resolution requirements

Dispute resolution MUST produce:

- a resolution decision (`fulfilled`, `failed`, or `refunded`)
- linked supporting evidence (`resolution_evidence_refs`)
- final settlement state transition (`fulfilled`, `failed`, or `refunded`)
- decision timestamp and responsible authority/service (`resolved_by`)

Unresolved open disputes SHOULD block normal routing for affected commitments according to network policy.

## Anti-monopoly safeguards

To reduce centralization risk, networks SHOULD avoid single-party control over all of:

- admission control
- routing policy
- settlement recording

At minimum, governance policy MUST disclose where these controls are held and how conflicts are mitigated.

## Conformance governance

Governance MUST publish:

- current required conformance profile
- profile retirement schedule
- compatibility matrix between protocol and profile versions
- rollout plan for mandatory upgrades

## Transparency and accountability

Networks SHOULD publish periodic governance and dispute metrics:

- active/restricted/revoked participant counts
- dispute open/close times
- appeal/reversal rates
- profile adoption percentages

This supports operator trust and policy calibration over time.
