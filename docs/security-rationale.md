# Security rationale

Covenant Layer is safer than direct-operation agent systems only if the security model is explicit.

## What it improves

The main improvement is that the edge agent does less direct deterministic fulfillment.

Instead of forcing the edge agent to:
- click through UIs
- invoke exact operations
- commit low-level side effects

the model shifts more exact execution to providers.

That reduces one dangerous class of risk:
the probabilistic-to-deterministic bridge at the edge.

## What it does not solve automatically

It does not make trust automatic.

The risk moves toward:
- authority abuse
- provider impersonation
- misleading offers
- weak evidence
- fulfillment mismatch
- replay problems
- broker manipulation
- unresolved disputes

The protocol must therefore make:
- authority
- acceptance
- evidence
- settlement
- dispute

first-class concepts.