# Example flows

This document gives concrete example flows for Covenant Protocol.

## Example 1: travel booking

### User request

A user tells an agent:

"Get me from New York to Paris next Thursday. Prefer nonstop if the price difference is reasonable. No overnight layovers. Business class only if it stays under $2,500. Show me the best two before booking."

### Step 1: objective

The agent creates an Objective with:
- destination and date
- layover constraints
- class preference
- budget cap
- approval requirement before purchase

### Step 2: authority grant

The user provides an Authority Grant that allows:
- search
- comparison
- temporary holds

The grant forbids:
- final purchase without approval

### Step 3: discovery

The agent sends the Objective to a broker.

The broker routes it to eligible providers:
- Provider A
- Provider B
- Provider C

### Step 4: offers

Provider A returns:
- nonstop
- $2,315
- short hold
- moderate refund terms

Provider B returns:
- one stop
- $1,940
- strong refund terms

Provider C returns:
- business class
- $2,480
- tighter change terms

### Step 5: comparison

The agent compares the tradeoffs and shows the user the best two options.

### Step 6: acceptance

The user approves Provider A.

The agent issues Acceptance referencing Provider A’s Offer.

At this point, Provider A is obligated under the accepted terms.

### Step 7: fulfillment

Provider A performs the actual booking through its own systems.

This may involve:
- airline APIs
- GDS systems
- internal reservation systems
- human operators
- internal automation

These details are outside protocol scope.

### Step 8: proof

Provider A submits Evidence Records:
- booking confirmation
- fare class
- ticket number
- refund terms reference

### Step 9: settlement

The Settlement Service issues a Settlement Receipt with state:
- fulfilled

## Example 2: failure and refund

This flow begins the same way as Example 1.

After Acceptance, the Provider attempts fulfillment but fails because inventory vanished before ticket issuance.

The Provider submits Evidence Records showing:
- accepted offer
- attempt timestamp
- failure reason
- no completed issuance

The Settlement Service marks the state:
- failed

If policy requires compensation or refund:
- a second Settlement Receipt may record
- refunded

## Example 3: dispute

A Provider claims fulfillment.

The agent or user disputes whether the fulfilled result actually matched the accepted terms.

Possible dispute reasons:
- wrong cabin class
- prohibited overnight layover
- refund terms not as promised
- delayed ticket issuance past the valid window

The commitment enters:
- disputed

The Verifier checks the evidence.

A later Settlement Receipt records the resolution:
- fulfilled
- failed
- refunded

## Example 4: provider counteroffer

A provider cannot satisfy the original objective exactly.

Instead of returning a normal Offer, it returns a counteroffer such as:
- same route
- later departure
- stronger refund terms
- lower price
- one stop instead of nonstop

The agent may:
- reject the counteroffer
- compare it with other offers
- present it to the user
- accept it if authority allows

A counteroffer does not create commitment by itself.

Only valid Acceptance does.

## Example 5: authority failure

The agent receives an attractive Offer but the Authority Grant does not allow final purchase.

The agent may:
- compare the offer
- show it to the user
- request approval

The agent may not:
- accept the offer as binding
- authorize fulfillment
- spend above the delegated limit

The protocol should treat this as a blocked action, not as a silent failure.