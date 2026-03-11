# Architecture

The architecture can be thought of as **layered**.

```text
+-----------------------------------------------------------+
| Layer 5: user intent                                      |
| goals, preferences, budget, constraints, approval rules   |
+-----------------------------------------------------------+
| Layer 4: coordination                                     |
| objectives, offers, acceptances, commitments              |
+-----------------------------------------------------------+
| Layer 3: trust                                            |
| authority, identity, evidence, receipts, disputes         |
+-----------------------------------------------------------+
| Layer 2: fulfillment                                      |
| provider execution, booking, payment, issuance            |
+-----------------------------------------------------------+
| Layer 1: infrastructure                                   |
| HTTP, queues, logs, internal APIs                         |
+-----------------------------------------------------------+
```

---

## Layer 1: infrastructure

This is the transport and systems foundation.

**Examples:**

- HTTP
- message queues
- logs
- internal APIs
- databases
- service meshes
- ledgers or transparency systems

This layer matters, but it is not where the core model lives.

---

## Layer 2: fulfillment

This is where exact execution happens.

**Examples:**

- ticket issuance
- booking flows
- payment capture
- refund processing
- policy enforcement
- internal workflow engines

**Providers own this layer.** This is where deterministic execution belongs.

---

## Layer 3: trust

This layer makes delegation and accountability possible.

**Examples:**

- authority grants
- identity
- evidence
- receipts
- dispute records
- revocation and expiry

Without this layer, commitments are hard to trust.

---

## Layer 4: coordination

This is the **center of the model**.

**Examples:**

- objectives
- offers
- acceptance
- counteroffers
- settlement transitions

This is where the public interface changes from procedures to commitments.

---

## Layer 5: user intent

This is the human side of the system.

**Examples:**

- goals
- preferences
- budgets
- constraints
- approval rules
- exceptions

Language models are strongest close to this layer.

---

## Design rule

The edge agent should **move upward** in the stack:

- closer to intent
- closer to coordination
- farther from exact execution

Execution does not disappear. It becomes increasingly **provider-owned**.

---

## Old architecture vs new architecture

**In the old architecture**, the agent spends too much of its energy at fulfillment and below:

- direct tool use
- browser control
- exact operations
- brittle side effects

**In the Covenant Layer model**, the agent spends more of its energy at:

- intent interpretation
- offer comparison
- policy application
- acceptance
- monitoring

That is the **architectural correction**.
