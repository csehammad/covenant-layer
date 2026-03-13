# Chain-Only E2E Demo

This demo shows an end-to-end Covenant flow with:

- one `agent_service` onboarding to `active`
- three providers onboarding to `active`
- objective publication
- **LLM-powered agent reasoning** for offer comparison (reasoning model)
- deterministic fallback scoring when no LLM is configured
- acceptance, evidence, and settlement artifact output
- HTML timeline visualization

All framework writes are chain-mode and require signed raw transactions.

## Prerequisites

- Rust toolchain installed
- Python 3 installed
- Base Sepolia funded wallets for demo actors
- deployed registry contract address
- signed raw transactions for each register/transition/attestation operation
- (optional) OpenAI or Anthropic API key for LLM-powered agent reasoning

## 1) Configure environment

Copy template and fill values:

```bash
cp demo/fixtures/chain-config.template.env demo/fixtures/chain-config.env
cp demo/fixtures/raw-tx.template.json demo/fixtures/raw-tx.json
```

Set environment from config:

```bash
set -a
source demo/fixtures/chain-config.env
set +a
export DEMO_RAW_TX_FILE="$(pwd)/demo/fixtures/raw-tx.json"
```

## 2) Configure agent reasoning (optional)

To enable LLM-powered offer comparison using a reasoning model, set your API key:

```bash
# OpenAI (default provider, uses o3-mini reasoning model)
export OPENAI_API_KEY="sk-..."

# Or Anthropic (uses Claude Sonnet with extended thinking)
# export DEMO_LLM_PROVIDER=anthropic
# export ANTHROPIC_API_KEY="sk-ant-..."

# Override the model if needed
# export DEMO_LLM_MODEL=o3-mini
```

When configured, the agent will:
1. Send the objective and all provider offers to the reasoning model
2. The model reasons step-by-step about tradeoffs (price vs convenience vs refund terms vs cabin class)
3. Produces a ranked recommendation with justification and confidence level
4. The selected offer feeds into the acceptance step

If no API key is set, the demo falls back to deterministic weighted scoring — the protocol flow still runs end-to-end, but without LLM judgment.

## 3) Run end-to-end demo

```bash
./demo/scripts/run_demo.sh
```

Generated artifacts:

- `demo/artifacts/onboarding.demo.json`
- `demo/artifacts/objective-flow.demo.json` — includes `agent_reasoning` block with model output, reasoning trace, and confidence
- `demo/artifacts/protocol-envelopes.demo.json`
- `demo/artifacts/demo.full.json`

## 4) View visualization

Serve repo root and open:

```bash
python3 -m http.server 8080
```

Open:

- `http://localhost:8080/docs/demo-e2e-timeline.html`

## Validation checklist

- agent status reaches `active`
- provider alpha/bravo/charlie statuses reach `active`
- scored offers include three providers
- `agent_reasoning_complete` event present with `selection_method` (`llm_reasoning` or `deterministic`)
- if LLM enabled: `agent_reasoning` block in artifact includes model, reasoning, ranked_offers, confidence
- selected offer exists
- evidence and settlement events appear in `objective-flow.demo.json`
# Demo

This demo proves one narrow claim:

An agent can publish an objective, receive competing outcome-level offers, accept one under delegated authority, and produce proof plus settlement.

## Vertical

Travel

## Components

- user app
- agent service
- authority service
- broker
- provider A
- provider B
- provider C
- evidence store
- settlement service
- network event service
- web dashboard

## Goal of the demo

Show that the edge agent does not directly operate every downstream system.

The agent coordinates.

The provider fulfills.

## Realtime architecture

Each actor runs as a separate process with its own HTTP API.

All services publish lifecycle events to the network service (`/publish`).
The dashboard subscribes to realtime events over SSE (`/events`).

### Layer tags used in events

- `intent`
- `coordination`
- `trust`
- `fulfillment`
- `infrastructure`

### Protocol stages used in events

- `objective`
- `authority`
- `offer`
- `acceptance`
- `fulfillment`
- `evidence`
- `settlement`
- `dispute`

## Run

From repo root:

```bash
python -m demo.run_demo
```

Then open:

- Dashboard: `http://127.0.0.1:8110`

Click **Start Travel Demo** in the dashboard to execute one full lifecycle.

### Quick smoke test

```bash
python -m demo.run_demo --once
```

This starts all entities, runs one objective, prints the accepted offer and event count, then exits.