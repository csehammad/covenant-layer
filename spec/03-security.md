# Security considerations

Covenant Protocol is not secure just because it has clean object names.

It only becomes safer than direct-operation agent systems if the model shifts risk in a controlled way.

The main architectural move is this:

- the edge agent does less direct deterministic fulfillment
- the provider does more exact execution

That is a real security improvement, but only if authority, commitment, fulfillment, evidence, and settlement are all explicit and hard to fake.

## What the protocol improves

The main security gain is that the edge agent is asked to do less exact execution in external systems.

Instead of forcing the edge agent to:
- click through UIs
- invoke exact operations
- commit low-level side effects
- improvise across arbitrary systems

the protocol shifts more exact execution to providers.

That reduces one dangerous class of risk:
the probabilistic-to-deterministic bridge at the edge.

The edge agent focuses on:
- understanding intent
- comparing offers
- applying policy
- getting approval
- monitoring outcomes

The provider focuses on:
- exact fulfillment
- exact booking or transaction execution
- proof
- accountability

That split is the main safety benefit of the protocol.

## What the protocol does not solve automatically

The protocol does not make trust automatic.

It does not guarantee that:
- an authority grant is valid
- a provider is honest
- an offer is realistic
- a broker is fair
- evidence is truthful
- fulfillment matched accepted terms
- disputes will resolve themselves

The protocol reduces one class of risk, but it shifts the center of gravity toward:
- authority abuse
- provider impersonation
- false or misleading offers
- weak evidence
- fulfillment mismatch
- replay and expiry failures
- broker manipulation
- privacy leakage
- unresolved disputes

## Threat model shift

In direct-operation agent systems, dominant risks include:
- wrong clicks
- wrong tool invocations
- state misreads
- prompt injection into browsing or tool contexts
- unintended side effects from approximate reasoning

In Covenant Protocol, those risks are reduced at the edge.

The dominant risks become:
- invalid or overbroad authority
- fraudulent providers
- malformed or deceptive offers
- unclear acceptance boundaries
- provider failure after acceptance
- fake or weak evidence
- incorrect settlement
- lack of dispute handling

This is a better security shape only if these risks are treated as core protocol concerns.

## Delegated authority

An agent must not act with vague or unlimited power.

An Authority Grant should be bounded by at least:
- scope
- time
- budget or exposure
- approval rules
- revocation path

A valid Authority Grant should answer:
- what may this agent do?
- what may it not do?
- for how long?
- on whose behalf?
- under what approval conditions?

If grants are too broad, the protocol simply moves the blast radius from low-level execution mistakes to commitment mistakes.

## Offer authenticity

A malicious actor may pretend to be a provider or broker.

Implementations must have a participant identity model strong enough to answer:
- who issued this offer?
- is this participant authorized to make this class of commitment?
- under what network or trust framework is this participant recognized?

Unsigned or weakly bound offers should not be eligible for acceptance.

## Acceptance semantics

The protocol must distinguish clearly between:
- viewing an offer
- ranking an offer
- tentatively selecting an offer
- accepting an offer

Acceptance is the critical security boundary in the protocol.

A valid acceptance creates commitment.

If this boundary is fuzzy, the protocol will recreate a major failure mode of current agent systems: turning approximate intent into premature side effects.

## Provider fulfillment responsibility

After valid acceptance, the selected provider is responsible for exact fulfillment.

This is the architectural move that helps solve the probabilistic-to-deterministic problem.

The edge agent is no longer expected to directly perform the low-level booking, issuance, or transaction itself.

The provider does.

That safety gain only holds if the provider can be held accountable for mismatch between:
- accepted terms
- actual fulfillment
- submitted proof

## Evidence quality

Evidence is not the same as truth.

A signed statement or receipt can prove that:
- a participant issued a claim
- the claim existed at a given time
- the claim was recorded

It does not automatically prove that the claim was correct.

For that reason, Evidence Records must be evaluated against:
- accepted offer terms
- domain rules
- timing requirements
- verification policy

Weak evidence should not produce strong settlement outcomes.

## Broker power and routing integrity

Brokers are powerful participants.

A broker may:
- suppress better offers
- prefer related providers
- alter ranking logic
- leak objective data
- steer flow for profit

Implementations should support:
- auditable routing policy
- traceable offer provenance
- clear distinction between broker recommendation and provider commitment

## Replay and expiry

Offers, acceptances, and evidence records may be replayed if freshness is weak.

Implementations should include:
- unique identifiers
- issuance timestamps
- expiry windows
- replay detection
- state-transition checks

An expired offer must not become valid through replay.
A revoked authority grant must not remain usable because of stale state.

## Privacy and selective disclosure

Participants should receive only the information needed for their role.

Providers should receive only the information needed to evaluate or fulfill an objective.
Brokers should receive only the information needed to route and compare.
Verifiers should receive only the information needed to validate evidence.

The protocol should not assume that because data is signed or logged, it is safe to disclose broadly.

## Dispute is a first-class outcome

The protocol must not assume that all conflicts can be resolved automatically.

A settlement service should support at least:
- fulfilled
- failed
- refunded
- disputed
- expired

Dispute is not an implementation detail.

It is part of the core security model.

## Security posture summary

Covenant Protocol is safer than direct-operation agent systems in one important way:

It reduces how often a probabilistic edge agent must directly produce deterministic side effects in external software.

That is the main benefit.

Instead of forcing the edge agent to act like an exact operator, the protocol lets the edge agent focus on:
- intent interpretation
- tradeoff analysis
- approval
- commitment selection
- fulfillment monitoring

and lets the provider focus on:
- exact execution
- exact fulfillment
- proof
- accountability

The protocol therefore improves security by reducing one dangerous class of risk.

But it does not eliminate risk.

It moves the center of gravity.

The key security question is no longer:

"Can the agent safely click the right button?"

It becomes:

"Can the network prove that the right party, under the right authority, made the right commitment, fulfilled the right outcome, and produced evidence strong enough to justify settlement?"