# Mental model

The central argument of Covenant Layer is that modern agents are being treated as software operators.

That means the current stack keeps trying to solve the same problem by adding more ways for an agent to directly act on software built for humans:

- tool calling
- code execution
- browser automation
- workflow packages
- runtimes and gateways
- supervision and approval layers

These are useful patches. They are not the final shape.

## The core mismatch

Language models are probabilistic systems.

They infer.  
They rank.  
They approximate.  
They guess.

Software systems are deterministic systems.

They expect:
- exact inputs
- exact permissions
- exact state transitions
- exact side effects

When an agent directly operates software, it has to bridge from:
- fuzzy intent
- approximate reasoning
- ranked options

to:
- exact operations
- exact commitments
- exact consequences

That bridge is where many current agent failures live.

## The correction

The correction is to stop making the edge agent carry all deterministic execution burden.

The edge agent should be strongest at:
- understanding intent
- comparing tradeoffs
- applying policy
- getting approval
- monitoring outcomes

Exact fulfillment should move toward providers that already own execution.

That is the conceptual move from procedures to commitments.

## The key inversion

In the old model, the agent tries to operate the software world itself.

In the Covenant Layer model, the agent:
- states the objective
- carries authority
- gathers competing offers
- accepts one
- monitors fulfillment

The provider:
- commits to the outcome
- performs the exact execution
- proves what happened
- absorbs mismatch responsibility

That is the shift.

## The practical rule

Where work is delegated, multi-step, and costly to get wrong, the public interface should increasingly move from procedures to commitments.

That does not mean tools disappear.

It means tools move down the stack and stop being the main public interface for agents.