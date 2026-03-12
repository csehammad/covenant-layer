# Governance Model

This framework assumes decentralized execution with explicit governance boundaries.

## Roles

- **Contract Deployer (bootstrap):** deploys initial registry contract.
- **Governance Multisig / DAO:** manages upgrades and emergency controls.
- **Independent Registrants:** self-register participants using objective rules.
- **Brokers and Agents:** consume eligibility state and enforce active-only routing policy.

## Upgrade policy

- All implementation upgrades require timelock delay.
- Upgrade proposals must publish:
  - rationale
  - compatibility impact
  - migration plan
- Emergency paths must be narrowly scoped and expire automatically.

## Role permissions matrix

| Action | Registrant | Governance Multisig/DAO | Broker/Agent Consumer |
|---|---|---|---|
| Submit registration manifest | yes | optional | no |
| Submit signed registration transaction | yes | optional | no |
| Move `requested -> identity_verified` | no | yes | no |
| Move `identity_verified -> conformance_passed` | no | yes | no |
| Move `conformance_passed -> probation` | no | yes | no |
| Move `probation -> active` | no | yes | no |
| Move `active -> restricted` | no | yes | no |
| Move `restricted -> revoked` | no | yes | no |
| Route traffic to participant | no | no | yes (active only) |

Governance should publish objective criteria for each transition to avoid discretionary gatekeeping.

## Anti-monopoly constraints

- Valid registration cannot require private approval by one operator.
- Contract events and state transitions must be public and queryable.
- Objective eligibility criteria should be documented and reproducible.

## Operational ownership split

- Chain consensus owns transaction ordering/finality.
- Governance controls contract evolution.
- Participants own their own keys, manifests, and operational endpoints.
- No single actor should control all of: key admission, routing, and settlement.

## Actor-specific onboarding policy hooks

- **Provider:** objective classes, evidence types, fulfillment accountability declaration, incident response policy, data protection policy, fulfillment SLA, idempotency window.
- **Broker:** routing scope, offer provenance policy, conflict-of-interest policy, audit retention, fairness review interval.
- **Settlement Service:** supported states, dispute transition policy, evidence link integrity policy, replay protection, finality wait.
- **Verifier:** verification domains, replay protection, decision policy, verification SLA.
- **Agent Service:** authority handling policy, approval gating requirement, acceptance guardrails.

## Agent activation controls

An `agent_service` must not be promoted to `active` unless governance confirms:

- conformance checks passed with approval gating enforced
- authority proof method and approval reference format are declared
- revocation check interval and max authority TTL are explicitly configured
- an agent policy attestation is attached in registry history

## Conformance profile policy

- Registrants must declare a supported conformance profile in manifest root.
- Current required value is `onboarding-v1`.
- Normative requirements are defined in `conformance-profiles/onboarding-v1.md`.
- Governance must publish migration timelines before changing required profile versions.
