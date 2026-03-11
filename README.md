# Covenant Layer

Covenant Layer is an open model for shifting agent systems from tool orchestration to outcome coordination.

The core claim is simple:

Modern agents are probabilistic systems being forced to operate deterministic software. That mismatch is where a lot of today’s fragility comes from. The long-term fix is not to keep teaching agents how to click better. It is to make software increasingly expose commitments about outcomes instead of only low-level procedures.

## Core idea

In the old model, the agent:
- calls tools
- runs scripts
- uses browser automation
- stitches workflows together
- directly carries too much execution burden

In the Covenant Layer model:
- the user states an objective
- the agent carries authority
- a broker or exchange routes that objective
- providers return offers they are willing to stand behind
- the agent compares and accepts one
- the provider fulfills the outcome
- the network records evidence and settlement

The key shift is from procedures to commitments.

## Vocabulary

### Covenant Layer
The broader shift from procedural interfaces to outcome coordination.

### Covenant Network
The ecosystem that forms when many participants adopt the model.

### Covenant Protocol
The formal rulebook that defines objectives, authority, offers, acceptance, evidence, settlement, and dispute.

## Repository layout

- `docs/` — essays, rationale, network model, architecture
- `spec/` — formal protocol draft
- `diagrams/` — text diagrams for the model
- `demo/` — proof-of-concept outline

## Status

This is an early working draft. The goal is to make the idea concrete enough to critique, implement, and improve.