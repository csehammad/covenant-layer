# Introduction

Covenant Protocol is the rulebook inside Covenant Layer.

It defines how participants coordinate around:
- objectives
- delegated authority
- offers
- acceptance
- fulfillment
- evidence
- settlement
- dispute

The protocol does not define provider internals.

It defines the public coordination model between participants.

## Purpose

The protocol exists to move agent systems upward from direct procedure execution to explicit outcome coordination.

Instead of asking an edge agent to directly operate every deterministic system involved in a task, the protocol creates a shared structure for:
- expressing what outcome is wanted
- proving who is authorized to act
- collecting provider commitments
- turning one accepted offer into an actual obligation
- recording evidence and settlement afterward

The main design move is simple:

- the agent coordinates
- the provider fulfills

That boundary is what makes the protocol different from ordinary tool orchestration.

## What this spec standardizes

This spec is about the common interface between participants.

It standardizes:
- the core roles in the network
- the main protocol objects
- the lifecycle from objective to settlement
- the meaning of acceptance
- the role of evidence and dispute

It does **not** attempt to standardize every internal implementation detail behind those steps.

## What is out of scope

The protocol is not a provider workflow engine.

It does not prescribe:
- how a provider books inventory internally
- how a broker ranks offers internally
- what software stack participants use
- whether fulfillment is done by APIs, humans, queues, browsers, or legacy systems

Those are implementation choices.

The protocol only cares about what must be visible and interoperable at the coordination layer.

## Why this matters

Current agent systems often force probabilistic models to directly cause deterministic side effects.

That creates a fragile bridge between:
- approximate reasoning
- exact execution

Covenant Protocol tries to improve that architecture by moving more exact execution responsibility to providers that are willing to stand behind explicit offers, while keeping the edge agent focused on:
- intent interpretation
- policy application
- offer comparison
- approval
- monitoring

This does not remove risk.

It changes where rigor must exist:
- authority must be explicit
- acceptance must be unambiguous
- fulfillment must be provable
- settlement must be recorded
- disputes must be first-class

## Reading guide

The rest of the spec is organized as follows:

- `01-problem.md` explains the mismatch the protocol is trying to solve
- `02-core-protocol.md` defines the roles, objects, lifecycle, and rules
- `03-security.md` explains how the threat model changes under this architecture
- `04-examples.md` shows the model in concrete scenarios
- `05-wire-integrity.md` defines canonicalization, signatures, replay controls, and error taxonomy
- `06-governance-dispute-upgrades.md` defines governance authority, dispute lifecycle, and upgrade policy
- `07-compatibility-matrix.md` defines version/profile compatibility and migration behavior

This introduction is intentionally high-level.

Its job is to define the frame:

The protocol is not about teaching agents to operate software more precisely.

It is about making software ecosystems expose commitments clearly enough that agents can coordinate outcomes without carrying the full burden of low-level execution.