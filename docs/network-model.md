# Network model

Covenant Layer becomes valuable when it forms a real network.

## Roles

The basic participants are:

- User
- Agent
- Broker
- Provider
- Verifier
- Settlement Service

## How the network forms

This does not begin as one giant public system.

It starts in narrow verticals where work is:
- delegated
- multi-step
- costly to get wrong
- full of policy and exception handling

Examples:
- travel
- logistics
- procurement
- claims processing
- vendor onboarding
- recruiting
- enterprise operations

## Role of each participant

### User
Defines objectives and approves commitments when required.

### Agent
Carries authority, interprets intent, compares offers, and monitors fulfillment.

### Broker
Routes objectives to eligible providers and aggregates offers.

### Provider
Makes offers and performs fulfillment after acceptance.

### Verifier
Checks evidence and validates claims about fulfillment.

### Settlement Service
Records final state such as fulfilled, failed, refunded, disputed, or expired.

## Why this is a network

This model is bigger than a single protocol document.

A protocol can define:
- message types
- lifecycle
- state changes
- security requirements

A network is what forms when many participants adopt those rules and interact repeatedly.

Without the protocol, the network is mush.

Without the network, the protocol is just a document.

## The practical shape

A mature Covenant Network would likely contain:

- users and organizations publishing objectives
- agents carrying delegated authority
- providers competing on commitment terms
- brokers routing and comparing offers
- verifiers checking evidence
- settlement services recording final state
- trust relationships between participants
- reputation and dispute handling over time

## Registration and onboarding

The network model assumes explicit participant admission rather than ad-hoc trust.

In practice, participants should not be routed production flow until they:
- publish a signed registration manifest
- pass conformance checks
- complete a probation period
- reach active status

The canonical protocol rules for this process are defined in:
- `spec/02-core-protocol.md` (Participant registration and onboarding)

The baseline lifecycle for participant admission is:
- requested
- identity_verified
- conformance_passed
- probation
- active
- restricted
- revoked

## Decentralization boundaries

To prevent registry and routing monopoly, the network should enforce clear boundaries:

- registration state is anchored in shared, publicly verifiable state (for example, chain-backed registry)
- no single operator privately controls admission, routing, and settlement simultaneously
- policy changes are governed through transparent governance processes
- eligibility resolution is reproducible from public state plus published policy

This keeps admission and operational trust auditable across organizations.

## Shared network services (minimum)

A production-grade network shape should provide at least:

- participant registry and status resolution
- manifest/signature verification tooling
- conformance profile publication and version history
- dispute recording and resolution interfaces
- evidence retention and integrity verification interfaces

Without these shared services, interoperability degrades into bilateral integrations.

## Main claim

The important shift is where the complexity lives.

In the current model, the edge agent carries too much of the burden:
- discovering tools
- sequencing operations
- recovering from failures
- deciding when state is safe to commit
- explaining what happened later

In the network model, more of that burden moves outward:
- providers declare what they can fulfill
- brokers route and compare
- verifiers check
- settlement services resolve

The world starts meeting the agent halfway.