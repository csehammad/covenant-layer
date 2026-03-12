# Core protocol

This document defines the core roles, objects, lifecycle, and state semantics of Covenant Protocol.

The protocol is not a provider-internal workflow engine. It is the public coordination model between participants.

## Normative language

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are to be interpreted as normative requirements.

Unless explicitly stated otherwise, implementation-specific behavior is allowed only if it does not violate required protocol semantics.

## Roles

### User
The principal whose objective is being pursued.

A User may:
- define an objective
- grant authority to an agent
- approve or reject offers
- dispute fulfillment

A User does not need to directly interact with every provider in the network.

### Agent
A system acting on behalf of a user or organization under delegated authority.

An Agent is responsible for:
- interpreting the objective
- carrying or presenting authority
- requesting offers
- comparing offers
- applying policy
- obtaining approval when required
- accepting an offer when authorized
- monitoring fulfillment
- escalating failures or disputes

An Agent is not required to directly perform low-level fulfillment.

### Broker
A routing and comparison participant.

A Broker is responsible for:
- receiving objectives
- identifying eligible providers
- distributing objectives or objective summaries
- collecting offers
- returning offers to the requesting agent
- optionally supporting negotiation or counteroffers

A Broker does not itself need to fulfill outcomes.

### Provider
A participant that can fulfill an outcome and is willing to stand behind an offer.

A Provider is responsible for:
- determining whether it can satisfy an objective
- issuing offers with stated terms
- performing fulfillment after acceptance
- producing evidence of fulfillment or failure
- accepting accountability for mismatch between accepted terms and actual result

A Provider may use any internal implementation it chooses.

### Verifier
A participant or service that checks evidence against protocol rules or domain rules.

A Verifier may:
- validate evidence format
- validate evidence timing
- compare evidence against accepted terms
- support dispute resolution

### Settlement Service
A participant or service that records final or intermediate state for an accepted commitment.

A Settlement Service is responsible for:
- recording state transitions after acceptance
- marking commitments fulfilled, failed, refunded, disputed, or expired
- linking settlement outcomes to evidence

## Core objects

### Objective

An Objective is a statement of desired outcome plus constraints, preferences, and approval conditions.

An Objective should include at least:
- objective identifier
- requesting principal identifier
- target outcome
- hard constraints
- soft preferences
- approval requirements
- expiry or freshness window

Examples of Objective content:
- destination and date
- budget cap
- prohibited conditions
- preferred timing
- required approval before purchase

The Objective is the starting point of the protocol.

### Authority Grant

An Authority Grant is a bounded delegation allowing an agent to act on behalf of a user or organization.

An Authority Grant should include at least:
- grant identifier
- issuer
- holder
- allowed actions
- forbidden actions
- budget or exposure limit
- approval requirements
- expiry
- revocation reference

Examples of Authority Grant semantics:
- may search
- may compare
- may hold
- may not purchase without approval
- may not exceed budget limit

The Authority Grant is what turns an agent from a helper into an authorized actor.

### Offer

An Offer is a provider’s proposed fulfillment under stated terms.

An Offer should include at least:
- offer identifier
- provider identifier
- referenced objective
- proposed outcome
- economic terms
- timing terms
- proof requirements
- expiration
- acceptance conditions

An Offer may include:
- hold windows
- refund conditions
- service-level guarantees
- eligibility conditions
- required user data before fulfillment

An Offer is informational until accepted.

### Acceptance

Acceptance is a valid statement that a specific Offer has been accepted by an authorized actor.

Acceptance should include at least:
- acceptance identifier
- referenced offer
- accepting actor
- timestamp
- approval reference, if required
- commitment activation terms

Acceptance is the critical boundary in the protocol.

Before Acceptance:
- an offer is a proposal

After valid Acceptance:
- the selected provider becomes responsible for fulfillment under stated terms

This is the main state transition that turns a possible outcome into an actual commitment.

### Evidence Record

An Evidence Record is a proof item related to fulfillment, failure, or dispute.

An Evidence Record should include at least:
- evidence identifier
- referenced commitment or acceptance
- submitting participant
- evidence type
- evidence payload or pointer
- timestamp
- integrity or signature mechanism

Examples of Evidence Record content:
- booking confirmation
- ticket number
- fulfillment timestamp
- refund record
- failure reason
- verification result

Evidence is not automatically truth, but it is the protocol-level basis for verification and settlement.

### Settlement Receipt

A Settlement Receipt is a record of final or intermediate state for an accepted commitment.

A Settlement Receipt should include at least:
- receipt identifier
- referenced commitment
- settlement state
- timestamp
- responsible service
- linked evidence references

The Settlement Receipt is how the network records the outcome of the commitment lifecycle.

## Object identity and integrity baseline

For interoperability, each protocol object SHOULD follow a common baseline:

- globally unique identifier
- issuer/participant identifier
- creation timestamp
- expiry or validity window where applicable
- deterministic hash over canonical object payload
- detached or embedded signature reference

At minimum, Offer, Acceptance, Evidence Record, and Settlement Receipt MUST be integrity-verifiable.

Unsigned, expired, or integrity-mismatched objects MUST be treated as invalid for state advancement.

## Settlement states

The minimum settlement states are:

- fulfilled
- failed
- refunded
- disputed
- expired

Implementations may define more detailed substates, but these are the required baseline states.

## Core lifecycle

The baseline lifecycle is:

objective
  -> discovery
  -> offer
  -> counteroffer
  -> accept
  -> fulfill
  -> prove
  -> settle
  -> dispute

### 1. Objective creation
A user or authorized upstream system creates an Objective.

### 2. Discovery
An agent or broker identifies eligible providers.

### 3. Offer submission
One or more providers return offers.

### 4. Counteroffer
An agent, broker, or provider may revise terms before commitment.

### 5. Acceptance
An authorized actor accepts one offer.

Acceptance creates commitment.

### 6. Fulfillment
The selected provider performs the actual domain action.

### 7. Proof
The provider submits evidence of fulfillment or failure.

### 8. Settlement
The settlement service records resulting state.

### 9. Dispute
If fulfillment is contested or terms were not met, the commitment may enter dispute.

## Protocol rules

### Rule 1: objective-first coordination
The protocol begins from an objective, not a low-level operation.

The requesting participant should not need to know every provider-internal method in advance.

### Rule 2: acceptance creates commitment
An Offer becomes a commitment only after valid Acceptance.

An unaccepted Offer is informational.
An accepted Offer is binding within the semantics of the participating network.

### Rule 3: provider fulfillment responsibility
After valid Acceptance, the selected Provider is responsible for performing the offered fulfillment according to stated terms.

This is the central shift of the protocol.

The edge Agent coordinates.
The Provider fulfills.

### Rule 4: fulfillment is provider-internal
The protocol does not constrain how a provider performs fulfillment internally.

A provider may use:
- APIs
- human operators
- internal software
- queues
- browser automation
- third-party systems

These are implementation details, not protocol requirements.

### Rule 5: evidence is required
A fulfilled or failed commitment should not disappear into private logs.

A provider is expected to submit domain-appropriate evidence sufficient for verification and settlement.

### Rule 6: settlement is explicit
The protocol should not leave final state implicit.

The outcome of an accepted commitment must be recorded through settlement state.

### Rule 7: dispute is first-class
Dispute is not an implementation detail.

If fulfillment is contested, the protocol must support an explicit disputed state and linked evidence.

## Participant registration and onboarding

The network must have an explicit process for admitting operational participants.

This section defines the minimum baseline for onboarding:
- Providers
- Brokers
- Verifiers
- Settlement Services
- Agent Services

### Registration object

Each participant type must publish a signed registration manifest before handling production flow.

The registration manifest should include at least:
- participant identifier
- participant type
- conformance profile identifier
- public signing keys and key validity window
- capability declaration
- supported objective or domain scope
- evidence profile and integrity method
- policy references for refund, dispute, and settlement behavior
- service interface handles used for protocol exchange
- manifest issuance and expiry timestamps

The canonical representation may be YAML or JSON, but:
- it must be canonicalized before signing
- signature verification must be deterministic
- expired manifests must be treated as invalid

Conformance profiles MUST be versioned.

For the framework baseline, manifests MUST declare:
- `conformance_profile: onboarding-v1`

Future breaking conformance changes MUST use a new profile identifier (for example, `onboarding-v2`).

### Registration state machine

The minimum registration states are:
- requested
- identity_verified
- conformance_passed
- probation
- active
- restricted
- revoked

Implementations may add substates, but these are the required baseline states.

### Onboarding rules

#### Rule 8: signed registration is required
A participant must not be treated as eligible for production routing unless it has a valid signed registration manifest.

Unsigned or unverifiable registrations are ineligible.

#### Rule 9: conformance before production flow
A participant must pass protocol conformance checks before entering active production status.

Conformance should include at least:
- schema correctness for protocol objects
- signature handling correctness
- lifecycle transition correctness
- evidence submission correctness

#### Rule 10: probation for new participants
Newly admitted participants should begin in probation with bounded traffic exposure.

Probation participants may be promoted only after meeting reliability and evidence quality thresholds defined by the participating network.

#### Rule 11: eligibility is state-bound
Only participants in active state are fully eligible for normal routing.

Restricted participants must have reduced or zero routing eligibility depending on policy.
Revoked participants must be ineligible.

#### Rule 12: registration changes are explicit
Capability changes, key rotation, policy updates, and status changes must be explicit updates linked to participant identity.

Silent mutation of participant registration state is non-compliant.

#### Rule 13: deterministic eligibility resolution
Routing participants (agent/broker) MUST resolve participant eligibility from explicit registration state.

At minimum:
- `active` is routable
- `probation` is bounded by policy
- `restricted` is constrained or blocked by policy
- `revoked` is not routable

Eligibility decisions SHOULD be explainable from recorded state and policy inputs.

### Provider-specific minimums

A Provider registration should include:
- offered domain categories
- objective classes supported
- major constraints supported
- fulfillment accountability declaration
- evidence types it can produce after acceptance

Providers without domain-appropriate evidence support should not be eligible for active status.

### Settlement service-specific minimums

A Settlement Service registration should include:
- supported settlement states
- dispute transition handling declaration
- evidence reference integrity method

A Settlement Service must not record fulfilled or failed outcomes without linked evidence references.

### Broker-specific minimums

A Broker registration should include:
- objective routing scope
- provider eligibility policy reference
- offer provenance handling declaration

A Broker should preserve traceability between returned offers and originating providers.

## Dispute and resolution baseline

Dispute handling MUST be explicit and stateful.

Minimum required behavior:

- any contested commitment can transition to `disputed`
- dispute reason and references MUST be recorded
- supporting and rebuttal evidence MAY be appended while disputed
- settlement must eventually transition to a terminal resolved state (`fulfilled`, `failed`, or `refunded`) unless policy explicitly allows open disputes

Networks SHOULD publish:
- dispute responder roles
- response-time objectives
- escalation and arbitration paths
- finality conditions for dispute closure

## Versioning and compatibility

To avoid hidden fragmentation:

- protocol object schemas SHOULD include version metadata
- conformance profile versions MUST be explicit
- compatibility policy MUST be published before incompatible changes are enforced

Implementations SHOULD fail closed when receiving unknown mandatory fields that alter commitment semantics.

Detailed wire/canonicalization/error requirements are defined in `05-wire-integrity.md`.

Detailed governance/dispute/upgrade requirements are defined in `06-governance-dispute-upgrades.md`.

Version and profile compatibility requirements are defined in `07-compatibility-matrix.md`.

## Why this file matters

This file is the center of the spec.

If `00-intro.md` explains the idea and `01-problem.md` explains why it exists, `02-core-protocol.md` must define what the system actually is.