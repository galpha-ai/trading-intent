# Paper Outline

## Working title

TIM: A Schema-Driven Intent Gateway for AI Trading Agents

## Core thesis

TIM decouples AI trading agents from execution backends by making intent schemas the single source of truth for capability discovery, validation, and dispatch.

## Research questions

1. How should an agent discover executable trading capabilities without hardcoded per-executor logic?
2. How can new intent types be added without changing parser code or agent prompts?
3. What system boundary cleanly separates agent reasoning from chain-specific execution?

## Core objects

- Intent schema: declarative definition of structure, constraints, templates, and shorthands.
- Generic parser: XML to JSON translation without intent-specific logic.
- Validator: enforcement of schema-level invariants.
- Dispatcher: routing from validated intent to executor endpoint.
- Executor: chain-specific implementation outside the TIM core.

## Suggested section structure

### 1. Introduction

- Motivate the agent-to-executor coupling problem.
- State the need for a stable intent interface.

### 2. Problem Statement

- Formalize the mismatch between agent-side planning and executor-side implementation.
- Explain why bespoke adapters do not scale across strategies and chains.

### 3. Design Principle

- Schema as single source of truth.
- Generic parsing, schema-driven validation, config-driven dispatch.

### 4. System Architecture

- Present the three-layer model.
- Walk through the runtime path from template discovery to executor response.

### 5. Extension Model

- Show that adding intent types and chains is data/config work, not core runtime surgery.
- Use the built-in intent examples as case studies.

### 6. Safety and Interface Invariants

- Required fields and constraint checking.
- Preservation of unknown XML elements for forward compatibility.
- Separation between validated intent structure and executor semantics.

### 7. Evaluation Plan

- Functional demonstration:
  add a new intent schema and show it appears in templates and validation without parser changes.
- Operational demonstration:
  configure multiple dispatch routes and show first-match routing by chain and type.
- Performance note:
  if performance claims are made, add explicit request-path benchmarks before publication.

### 8. Limitations

- XML was chosen as the current agent-facing interchange format; this is a design choice, not a universal claim.
- Executor semantics remain external to TIM.
- No empirical benchmark claims should be made yet without dedicated measurement.

### 9. Conclusion

- Re-state TIM as a minimal contract layer between planning agents and execution systems.

## Evidence checklist

- Existing code references:
  `src/schema/`, `src/intent/`, `src/dispatch/`, `docs/design/architecture.md`
- Existing intent examples:
  `intents/immediate.yaml`, `intents/limit_order.yaml`, `intents/conditional_entry.yaml`
- Existing demo surfaces:
  `examples/agent_demo.py`, `examples/echo_executor.py`

## Postpone

- Full prose draft
- Related-work section
- Experimental tables
- Benchmark section until measurements exist
