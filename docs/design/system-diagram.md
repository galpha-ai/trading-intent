# TIM System Diagram

This diagram is the paper-facing view of the current repository architecture. It stays intentionally close to the implementation and existing design docs.

```mermaid
flowchart LR
    subgraph Schema["Layer 1: Schema"]
        YAML["Intent YAML Schemas<br/>intents/*.yaml"]
    end

    subgraph Template["Layer 2: Agent Interface"]
        Templates["Template Generation<br/>GET /api/v1/templates"]
    end

    subgraph Runtime["Layer 3: Runtime Path"]
        Parser["Generic XML Parser<br/>XML -> JSON"]
        Validator["Schema Validator<br/>constraints + shorthands"]
        Dispatcher["Dispatcher Matcher<br/>intent_type + chain_id"]
        Sender["HTTP Sender"]
    end

    Agent["AI Agent"]
    Config["Dispatcher Config<br/>config/local.yaml"]
    Executor["Executor Service"]

    YAML --> Templates
    YAML --> Validator
    Agent -->|"discover capabilities"| Templates
    Templates -->|"XML template + field docs"| Agent
    Agent -->|"POST /dispatch with XML"| Parser
    Parser --> Validator
    Validator --> Dispatcher
    Config --> Dispatcher
    Dispatcher --> Sender
    Sender --> Executor
    Executor -->|"execution result"| Agent
```

## Diagram invariants

- Schema remains the source of truth for both agent-facing templates and runtime validation.
- The XML parser is generic; adding a new intent type should not require parser changes.
- Dispatch stays config-driven and executor-agnostic.
