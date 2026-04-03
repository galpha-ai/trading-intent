# TIM — Trading Intent Model

Schema-driven intent gateway for AI trading agents. Define intent formats as YAML schemas, serve fillable XML templates to agents, validate and dispatch to your execution services.

```
                  ┌────────────────────────────────────┐
                  │               TIM                    │
AI Agent ──XML──▶│ parse → validate(schema) → dispatch  │──▶ Executor
                  │                                      │
Agent ◀──────────│ GET /templates ← schema-generated     │
                  └────────────────────────────────────┘
```

## Why TIM?

As AI agents become a mainstream interface for on-chain trading, a critical gap has emerged: **there is no standard way for agents to express trading intents**. Every agent-to-executor integration is a bespoke, fragile bridge. Adding a new chain means rewriting agent logic. Adding a new strategy means rewriting executor contracts. The coupling is expensive, error-prone, and doesn't scale.

**TIM decouples agents from executors** by introducing a schema-driven intent layer between them. A single YAML file defines an intent type's structure, validation rules, agent-facing templates, and dispatch routing — all at once. Agents don't need to know how trades are executed. Executors don't need to know how agents think. TIM is the contract between them.

| What | How |
|------|-----|
| Define a new intent type | Add a YAML file to `intents/` |
| Agent discovers capabilities | `GET /api/v1/templates` |
| Agent learns the format | `GET /api/v1/templates/IMMEDIATE` → XML template + field docs |
| Validate without executing | `POST /api/v1/validate` |
| Execute | `POST /api/v1/dispatch` → routes to your executor |
| Add a new chain | Add a dispatcher line in config |
| Swap executor | Change the endpoint URL |

### Key Design Principles

- **Schema as single source of truth** — one YAML file drives validation, templates, docs, and routing. No duplication, no drift.
- **Generic XML parser** — the parser never changes when you add intent types. It converts any XML tree to JSON; the schema handles the rest.
- **Executor-agnostic** — TIM doesn't know or care how trades are executed. It validates structure and routes payloads. Your executor handles the chain-specific logic.
- **Forward-compatible** — unknown XML elements are preserved, not rejected. Agents and executors can evolve independently.

## Quick Start

```bash
# Start TIM
cargo run --bin tim_server

# Start the echo executor (in another terminal)
pip install flask && python examples/echo_executor.py

# Agent discovers what it can do
curl http://localhost:8080/api/v1/templates

# Agent gets the template for IMMEDIATE trades
curl http://localhost:8080/api/v1/templates/IMMEDIATE

# Agent fills in the template and dispatches
curl -X POST http://localhost:8080/api/v1/dispatch \
  -H "Content-Type: application/json" \
  -d '{"intent": "<intent><type>IMMEDIATE</type><chain_id>solana:mainnet-beta</chain_id><entry><condition><immediate>true</immediate></condition><action><buy><amount>0.1</amount><quote>So11111111111111111111111111111111111111112</quote><base>EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v</base></buy></action></entry></intent>"}'
```

### Agent Demo

A complete Python example showing the full agent workflow — discover templates, fill, validate, and dispatch:

```bash
pip install requests
python examples/agent_demo.py
```

See [examples/agent_demo.py](examples/agent_demo.py) for the full source.

## Intent Schemas

Schemas live in `intents/` as YAML files. Each schema defines:
- **fields** — structure, types, constraints, descriptions
- **template** — default XML template with `{{placeholder}}` slots
- **template_variants** — named variations (buy, sell_percentage, sell_all, ...)
- **xml_shorthands** — convenience expansions (e.g. `<amount>all</amount>` → percentage 100%)

### Built-in Intent Types

| Type | Description | Key Fields |
|------|-------------|------------|
| `IMMEDIATE` | Execute a token swap immediately on-chain | buy/sell action, `max_slippage_percent`, `callback_url` |
| `LIMIT_ORDER` | Place a limit order that triggers at a target price | `limit_price`, `price_direction` (above/below), `time_in_force` (GTC/FOK/IOC) |
| `CONDITIONAL_ENTRY` | Register a rule that triggers a trade on an event | `event_trigger` (news, listings, social), buy/sell action, `criteria` |

All intent types support optional `callback_url` for executor-to-agent notifications and optional `exit` strategies (take-profit / stop-loss).

### Schema Example

```yaml
# intents/immediate.yaml
name: IMMEDIATE
description: "Execute a token swap immediately on-chain"
fields:
  chain_id:
    type: string
    required: true
    description: "Target blockchain in CAIP-2 format"
  entry:
    type: object
    required: true
    fields:
      action:
        type: object
        required: true
        one_of:
          - buy:
              fields:
                amount: { type: number, required: true, min_exclusive: 0 }
                quote:  { type: string, required: true }
                base:   { type: string, required: true }
          - sell:
              fields:
                amount:     { type: number }
                percentage: { type: number, min: 0, max: 100 }
              constraints:
                - exactly_one_of: [amount, percentage]
  max_slippage_percent:
    type: number
    min: 0
    max: 100
  callback_url:
    type: string
```

### Schema Constraint Reference

| Constraint | Applies To | Example |
|------------|-----------|---------|
| `required` | any | `required: true` |
| `type` | any | `type: string` / `number` / `boolean` / `object` / `array` |
| `min`, `max` | number | `min: 0, max: 100` |
| `min_exclusive`, `max_exclusive` | number | `min_exclusive: 0` (must be > 0) |
| `enum` | string | `enum: [GTC, FOK, IOC]` |
| `pattern` | string | `pattern: "^0x[0-9a-fA-F]{40}$"` |
| `one_of` | object | Exactly one variant key must be present |
| `exactly_one_of` | object | Exactly one of the listed fields must be present |
| `items` | array | Schema applied to each array element |

**To add a new intent type**: create `intents/my_type.yaml`, restart TIM. The new type appears in `/templates`, validates automatically, and dispatches to any matching executor.

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/templates` | List available intent types with template summaries |
| GET | `/api/v1/templates/{type}` | Get XML templates + field descriptions for a type |
| POST | `/api/v1/dispatch` | Parse XML → validate → dispatch to executor |
| POST | `/api/v1/validate` | Parse XML → validate → return structured JSON |
| POST | `/api/v1/parse` | Parse XML → return JSON (no validation) |
| GET | `/api/v1/dispatchers` | List configured dispatch routes |
| GET | `/health` | Health check |

## Dispatch Configuration

```yaml
# config/local.yaml
dispatchers:
  - match: { intent_type: IMMEDIATE, chain_id: "solana:*" }
    endpoint: "http://solana-executor:3001/execute"
    timeout_secs: 30
  - match: { intent_type: IMMEDIATE, chain_id: "eip155:*" }
    endpoint: "http://evm-executor:3002/execute"
  - match: { intent_type: CONDITIONAL_ENTRY }
    endpoint: "http://rule-engine:3003/rules"
  - match: { intent_type: LIMIT_ORDER }
    endpoint: "http://order-engine:3004/orders"
    timeout_secs: 10
```

Matched top-to-bottom, first match wins. Glob patterns on `chain_id`.

## Building an Executor

An executor is any HTTP service that accepts TIM's JSON payload and returns a result:

```python
@app.route('/execute', methods=['POST'])
def execute():
    payload = request.json
    # payload["intent_type"], payload["chain_id"], payload["payload"]["entry"]["action"]
    return jsonify({"status": "confirmed", "transaction_hash": "..."})
```

See [Dispatch Protocol](docs/reference/dispatch-protocol.md) for the full spec.

## Project Structure

```
tim/
├── intents/             ← Intent schemas (YAML) — source of truth
│   ├── immediate.yaml
│   ├── limit_order.yaml
│   └── conditional_entry.yaml
├── src/
│   ├── schema/          ← Schema engine (load, validate, template)
│   ├── intent/          ← XML parser (generic recursive, depth/size limited)
│   ├── dispatch/        ← Routing engine
│   ├── http/            ← API surface
│   └── config.rs
├── config/local.yaml    ← Server + dispatcher config
└── examples/
    ├── echo_executor.py ← Minimal executor for testing
    └── agent_demo.py    ← Full agent workflow demo
```

## Documentation

- [Architecture](docs/design/architecture.md)
- [Intent XML Schema](docs/reference/intent-xml-schema.md)
- [Dispatch Protocol](docs/reference/dispatch-protocol.md)

## Use Cases

### AI Agent Trading Infrastructure

TIM is the natural integration point for any AI agent that needs to trade on-chain. Instead of hardcoding trade formats, agents query TIM's `/templates` endpoint at startup, learn the available intent types and their XML structure, fill in the template, and dispatch. When a new intent type is added (limit orders, DCA, liquidation), **every connected agent gains the capability automatically** — no agent-side code changes needed.

```
Agent prompt: "You can execute trades via TIM. Call GET /templates to see what's available."
→ Agent discovers IMMEDIATE, CONDITIONAL_ENTRY
→ Agent fetches XML template for IMMEDIATE
→ Agent fills in token addresses, amount, chain
→ Agent POSTs to /dispatch
→ TIM validates, routes to correct executor
```

### Multi-Chain Execution Gateway

Route Solana trades to one executor, EVM trades to another, Sui to a third — all through a single TIM gateway. Chain-specific logic stays in executors; TIM is chain-agnostic. Adding a new chain is a one-line config change:

```yaml
dispatchers:
  - match: { intent_type: IMMEDIATE, chain_id: "solana:*" }
    endpoint: "http://solana-executor:3001/execute"
  - match: { intent_type: IMMEDIATE, chain_id: "eip155:*" }
    endpoint: "http://evm-executor:3002/execute"
  - match: { intent_type: IMMEDIATE, chain_id: "sui:*" }
    endpoint: "http://sui-executor:3004/execute"    # ← new chain, one line
```

### Event-Driven Trading Rules

The `CONDITIONAL_ENTRY` intent type enables event-triggered trading: register rules that fire when news signals arrive, tokens get listed, or specific social media accounts post. TIM validates the rule structure and routes it to your rule engine. Combine with natural-language criteria for expressive conditions:

```xml
<intent>
  <type>CONDITIONAL_ENTRY</type>
  <chain_id>solana:mainnet-beta</chain_id>
  <entry>
    <condition>
      <event_trigger>
        <event_type>news_signal</event_type>
        <platform>x</platform>
        <author_handle>@elonmusk</author_handle>
        <criteria>mentions any memecoin ticker with positive sentiment</criteria>
      </event_trigger>
    </condition>
    <action>
      <buy>
        <amount>0.5</amount>
        <quote>So11111111111111111111111111111111111111112</quote>
        <base>EVENT_DERIVED</base>
      </buy>
    </action>
  </entry>
</intent>
```

### Rapid Executor Development

Any team can build a TIM executor — it's just an HTTP endpoint that receives validated JSON and returns a result. The 15-line [echo executor](examples/echo_executor.py) demonstrates the full contract. This makes it easy to:

- **Prototype** — spin up a mock executor in minutes to test agent integration
- **Specialize** — build chain-specific executors (DEX aggregator, orderbook, AMM) behind the same TIM interface
- **Swap** — replace an executor by changing one URL in config, zero downtime for agents

### Trading Platform Backend

TIM serves as a ready-made backend for trading platforms that want to add AI agent support. Instead of building custom agent APIs, deploy TIM as the agent-facing layer:

```
┌──────────┐     ┌─────┐     ┌──────────────────┐
│ AI Agent │────▶│ TIM │────▶│ Your Trading      │
│ (Claude, │     │     │     │ Engine / DEX      │
│  GPT,    │     │     │     │ Aggregator / etc. │
│  Custom) │     │     │     │                   │
└──────────┘     └─────┘     └──────────────────┘
```

TIM handles format negotiation, validation, and routing. Your engine receives clean, validated payloads in a consistent format regardless of which agent sent them.

### Intent Auditing and Compliance

Because every intent passes through TIM with a UUID, timestamp, and full XML/JSON payload, TIM naturally serves as an **audit point**. Log dispatched intents for:

- Compliance reporting (what did the agent trade, when, on which chain?)
- Agent behavior analysis (are agents staying within intended parameters?)
- Debugging (replay the exact XML an agent submitted)

## License

MIT
