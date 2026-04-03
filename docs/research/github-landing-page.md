# TIM: Standardize Meaning Before You Execute Money

## What TIM Is

TIM, the Trade Intent Model, is a semantic interface for trading systems.

It turns ambiguous human requests such as:

- `get me upside into the close`
- `hedge this if the market turns`
- `buy the best expression of this thesis across venues`

into a precise, machine-verifiable, cross-venue representation of economic action.

That representation is the missing layer between natural language and high-privilege execution.

## Why This Matters

Modern trading flows are fragmented across:

- broker APIs
- centralized exchanges
- DeFi protocols
- prediction markets
- on-chain wallets and transaction routers
- options, perps, spot, and structured multi-leg positions

Without a stable semantic layer, the same user request gets translated manually into many venue-specific actions. That makes execution brittle, hard to audit, and dangerous to automate.

## The Core Idea

TIM treats trading as a compilation problem:

**Natural language -> Trade intent -> Execution tasks and policies -> Venue-specific orders and transactions**

The key point is not just better parsing.

The key point is separation of concerns:

- the intent layer defines economic meaning
- the execution layer decides how to realize that meaning
- venue adapters translate the plan into local execution syntax

This is what makes the system debuggable, composable, and extensible.

## What TIM Enables

- Semantic precision before execution
- Cross-venue interoperability
- Portfolio-level strategy composition
- Liquidity-aware routing and execution decomposition
- Safer agent workflows for finance

## What This Repo Does Today

The current repository is the semantic contract layer.

Today it implements:

- intent schemas
- agent-facing templates
- parsing
- validation
- dispatch

It does **not** yet implement the full high-level planning stack, strategy synthesis engine, or low-level execution optimizer described in the broader TIM vision.

## What Comes Next

Future layers built on top of this contract surface can include:

- high-level portfolio planning
- strategy synthesis
- execution-task decomposition
- low-level execution-policy search
- microstructure-aware execution improvement

## Why Now

Two trends make this urgent:

1. More trading requests now arrive through chat, voice, and vibe-trading style instructions.
2. More users are connecting general-purpose agents directly to wallets, broker APIs, and DeFi protocols.

That means ambiguous language is increasingly becoming a direct execution surface.

## The Long-Term Vision

The end state is not a prettier brokerage UI.

The end state is an agent workflow stack where a user provides only an idea, the system synthesizes a candidate strategy, validates it, decomposes execution, and deploys it with one click.

TIM is the semantic backbone that makes that workflow possible.

## In One Sentence

TIM is a domain-specific language for compiling natural-language trading intent into verifiable, cross-venue execution plans.
