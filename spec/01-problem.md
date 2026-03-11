# Problem statement

Current agent systems keep forcing probabilistic models to directly operate deterministic systems.

That creates a brittle bridge between:
- approximate reasoning
- exact execution

The current market response has mostly been additive:
- more tools
- more code execution
- more browser control
- more supervision
- more workflow packaging
- more runtime state

These help. They do not change the basic mismatch.

## Design goal

Move the public interface upward from procedures to commitments.

The edge agent should:
- interpret
- compare
- approve
- monitor

Providers should:
- execute
- fulfill
- prove
- absorb mismatch liability

## Core design goals

- objective-first interaction
- bounded delegation
- explicit commitment semantics
- verifiable fulfillment
- clear settlement states
- provider freedom internally
- narrow, implementable core