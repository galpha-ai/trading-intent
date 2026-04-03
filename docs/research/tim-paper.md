# Trade Intent Models: A Semantic Interface for Cross-Market Trading and Agentic Strategy Execution

Bill Sun and Alpha.Dev team (https://alpha.dev/)

## Abstract

Trading systems remain fragmented by asset class, venue, execution model, and technical stack. A single economic idea may need to be implemented across broker APIs, centralized exchanges, decentralized protocols, options venues, prediction markets, and on-chain transaction flows, each with its own syntax and operational assumptions. This paper argues that trade intent should be treated as a first-class semantic layer for trading systems. A trade intent model is a domain-specific language that compiles ambiguous human requests into a precise, machine-verifiable, execution-agnostic representation of desired economic action. Its role is analogous to semantic parsing in search or modeling languages such as CVXPY in optimization: the user specifies what should be achieved; the system compiles that specification into a canonical representation; downstream components translate that representation into executable instructions. The central claim is that a well-designed trade intent model creates value along four dimensions: semantic precision, cross-venue interoperability, strategy composability, and agentic automation. More importantly, it serves as the representation layer for a broader action-oriented workflow in which a user provides only an idea, the system synthesizes a strong candidate strategy, and the platform deploys that strategy with one click.

## 1. Introduction

The core problem is simple: human trading intent is high-level, but execution infrastructure is fragmented and low-level. Users think in terms of outcomes, exposures, hedges, and constraints. Venues expose order types, product identifiers, chain-specific transaction formats, routing rules, and incompatible APIs. Directly mapping one into the other produces brittle systems.

This gap is becoming more acute because the interface to trading is changing. On the human side, more requests now arrive through chat, voice, and what might be called vibe trading: compressed instructions such as `get me upside into the close`, `hedge this if the market turns`, or `buy the best expression of this thesis across venues`. These are natural forms of human communication, but they are not safe execution formats. They are rich in intent and poor in specification.

On the machine side, a growing number of users are connecting general-purpose or non-specialist agents to wallets, broker APIs, and DeFi protocols. These agents can read natural language, call tools, and execute multi-step financial actions, but they often lack a stable semantic layer that separates user meaning from irreversible financial execution. In practice, unstructured prompts become direct action surfaces.

Recent incidents show why this matters. In February 2026, Lobstar Wilde, an autonomous crypto agent built on OpenClaw, misinterpreted a small-transfer request and sent about 52.4 million LOBSTAR tokens to a stranger. In March 2026, the National Internet Finance Association of China warned that OpenClaw-style financial deployments could expose sensitive credentials and trigger erroneous transfers or unintended purchases.

These examples point to a common systems problem: the missing layer is often not prediction or contract logic, but the absence of a rigorous intermediate representation between ambiguous language and high-privilege action.

This mismatch creates three failures. Natural language is expressive but ambiguous. Execution interfaces are precise but siloed. Automation without a stable intermediate representation is hard to audit and hard to debug. When something goes wrong, it is often unclear whether the system misunderstood the user’s goal, constructed the wrong strategy, or translated the strategy incorrectly into venue-native instructions.

The proposed compilation pipeline is:

**User idea or natural-language instruction -> High-Level Trade Intent -> Execution Tasks and Policies -> Venue-Specific Orders and Transactions**

This framing shifts the goal from better order entry to a semantic interface for action.

Consider the following natural-language request:

`Buy $10 of NVDAx on Solana, $5 of PAX Gold on the Ethereum L2 with the best available liquidity between Base and Arbitrum, buy one Yes contract on "Can NVDA exceed $200 in February 2026?", buy one contract on "2 cuts" for "How many times will the Fed cut this year?", and establish an AAPL June 6 190/200 bull call spread through Alpaca via MCP.`

This request is economically coherent but operationally heterogeneous. A trade intent model allows the system to treat it as a single structured trade vector rather than a bag of unrelated API calls.

## 2. What Is a Trade Intent Model?

A trade intent model is a formal intermediate representation of desired economic action. It is best understood as a domain-specific language for trading semantics. Its purpose is not merely to record orders, but to express what the user wants in a form that is precise, canonical, execution-agnostic, and machine-operable.

The schema should represent economic meaning rather than venue syntax. A useful intent object captures the economic objective, instruments and action types, composition structure, constraints, execution preferences, and provenance across parsing, validation, and lowering. This is what allows similar requests to map to similar structures and allows downstream components to reason about the request algorithmically rather than instruction by instruction.

## 3. Why Trade Intent Models Matter

Trade intent models create value along four dimensions.

1. Semantic precision and debuggability. By forcing a translation step from human expression into a canonical schema, the system can detect underspecification, contradictions, or infeasible requests before any trade is placed.
2. Cross-venue interoperability. Spot, options, perps, prediction contracts, broker flows, and on-chain actions can be represented through one semantic interface even when the final execution backends remain heterogeneous.
3. Strategy composability. Once a request is represented canonically, multi-leg packages, routing alternatives, cost-risk tradeoffs, and portfolio-level constraints can be reasoned about jointly.
4. Agentic automation. Parsers, validators, simulators, execution planners, monitors, and agents can interoperate through one shared object instead of brittle prompt glue or venue-specific adapters.

Without an intermediate representation, these benefits collapse into black-box automation.

## 4. Hierarchical Execution

Execution itself should be layered. The right analogy is robotics: high-level planning is distinct from low-level motor execution. Trading systems should make the same distinction between high-level trade intent and low-level execution tasks or execution policies. In the context of this paper, that layered execution stack should be read as an extension of the current repository rather than as a claim about what the present codebase already implements.

High-level trade intent is the planning layer. It captures the economically meaningful objective in a way that can be derived from natural language without leaving economically relevant ambiguity. It is also the right surface for cross-venue reasoning, liquidity aggregation, and policy-aware routing. A request such as `buy 100,000 shares of AAPL before the close` or `buy $1 million of NVDA call options this week` should first be represented as a portfolio-level object with explicit notional, time horizon, impact tolerance, urgency, and venue constraints. Only then should it be decomposed into lower-level execution tasks.

Those lower-level tasks belong to an execution layer. A single high-level portfolio trade may expand into many child orders, each governed by a different execution policy. Depending on market structure, those tasks may be realized through deterministic execution algorithms such as TWAP and VWAP, through broker-native smart routing, through on-chain naive slicing, through AMM or aggregator-based execution, or through richer low-latency strategies that incorporate market-structure or macro-sensitive execution signals. These are not different user intents. They are different compilations of the same intent into market-specific execution orders.

This separation also creates a principled place for microstructure-level improvement. The same intent can be executed better by blending in low-latency tactics, order-book-aware placement, queue-sensitive routing, microstructure alpha, or on-chain route-selection and timing improvements. The semantic layer is responsible for economic correctness; the execution layer is responsible for realized execution quality. The user specifies the portfolio objective once at the natural-language level, and the system remains free to search over better low-level implementations without changing that original intent. The current GitHub repository does not yet include this planning-and-execution hierarchy; it motivates it.

## 5. A System View: From Idea to Agentic Execution

The long-term objective is not a thin brokerage interface. It is an action-description layer and agent workflow substrate in which a user provides an idea, the system synthesizes a strong candidate strategy, validates it, and deploys it with one click.

1. Idea ingestion. The user supplies a thesis, desired position, hedge objective, or multi-step action request.
2. Semantic parsing into high-level trade intent. The request is normalized into a canonical, machine-verifiable object that supports cross-venue reasoning and liquidity aggregation.
3. Strategy synthesis and task decomposition. The system generates candidate implementations and breaks large or time-constrained portfolio intents into executable tasks and child orders.
4. Validation and policy checks. The platform checks feasibility, liquidity, risk, permissions, and portfolio consistency.
5. Low-level execution policy selection. The system chooses concrete execution methods such as TWAP, VWAP, smart routing, AMM-aware splitting, or low-latency microstructure-informed enhancement.
6. Execution compilation and deployment. The selected strategy is lowered into venue-specific orders, transactions, or MCP workflows and run on an agent platform.

The trade intent schema is the stable interface across these stages. The accompanying GitHub repository, `galpha-ai/intent-kit` (`https://github.com/galpha-ai/intent-kit`), should be read as the current reference implementation of the contract layer in this architecture: schema definition, template generation, parsing, validation, and dispatch. That stability also makes incremental open development possible: parsers, schemas, validators, simulators, adapters, and agents can evolve independently while remaining compatible through the shared representation.

## 6. Design Principles for a Practical Trade Intent DSL

A practical trade intent language should satisfy five requirements.

1. Semantic completeness. It must represent portfolios, conditional logic, dependencies, and nontrivial execution preferences rather than only single-venue orders.
2. Canonical structure. Superficially different phrasings of the same request should map to closely related forms.
3. Separation of concerns. The intent layer should capture what the user wants economically; later layers should decide how best to implement it.
4. Verifiability. Each translation stage should be auditable, with preserved provenance from request to parsed intent to selected strategy to final execution artifacts.
5. Extensibility. New instruments, venues, chains, and action types should be incorporable without redesigning the entire representation.

A wrapper translates syntax. A real trade intent model organizes semantics.

For clarity, the present repository implements only part of the full stack proposed in this paper. It currently covers the semantic gateway layer: intent schemas, agent-facing templates, parsing, validation, and dispatch. High-level portfolio planning, strategy synthesis, execution-policy search, and low-level algorithmic execution are future layers that can be built on top of that interface.

## 7. Open Problems

Several research problems remain open: selecting the right abstraction level for the schema; preserving semantic meaning during lowering; deciding which missing fields can be inferred safely; handling partial fills and cross-venue failure recovery; and constraining agent autonomy through risk, policy, and audit layers. These are not peripheral concerns. They are core design constraints for any deployment-grade trade intent system.

## 8. Conclusion

Trade intent models provide a semantic interface between human trading ideas and heterogeneous execution infrastructure. They transform ambiguous input into a precise, machine-verifiable, and execution-agnostic representation of economic action. Their value is broader than cleaner order syntax. A well-designed trade intent layer enables cross-venue interoperability, makes multi-leg strategies composable, improves debuggability, and supports a principled separation between high-level economic intent and low-level execution policy. In that sense, the trade intent model should be viewed not as a feature, but as infrastructure: the shared semantic substrate that allows fragmented markets, heterogeneous APIs, and agent platforms to interoperate through one language of economic intent.
