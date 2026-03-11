# Core protocol

This document defines the core roles, objects, lifecycle, and state semantics of Covenant Protocol.

The protocol is not a provider-internal workflow engine. It is the public coordination model between participants.

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

## Why this file matters

This file is the center of the spec.

If `00-intro.md` explains the idea and `01-problem.md` explains why it exists, `02-core-protocol.md` must define what the system actually is.