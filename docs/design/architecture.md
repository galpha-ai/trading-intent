# Architecture

## Three-Layer Design

```
Layer 1: Schema     YAML files define intent types, fields, constraints
                    ↓ drives ↓
Layer 2: Template   Agent-facing XML templates + field descriptions
                    ↓ drives ↓
Layer 3: Dispatch   Route validated intents to executor endpoints
```

**Schema is the source of truth.** Everything else — parsing, validation, templates, dispatch — is derived from it.

## Data Flow

```
1. Agent calls GET /templates/IMMEDIATE
   → TIM returns XML template + field descriptions (from schema YAML)

2. Agent fills in template, sends POST /dispatch with XML

3. TIM parses XML → JSON (generic recursive parser)
   → Applies configured shorthands (e.g. <amount>all</amount>)

4. TIM validates JSON against schema YAML
   → Checks required fields, types, numeric constraints, one_of, exactly_one_of

5. TIM matches dispatcher by (intent_type, chain_id) glob pattern

6. TIM POSTs ValidatedIntent JSON to executor endpoint

7. Executor response forwarded to agent
```

## Module Structure

```
src/
├── schema/           # Schema engine — THE core of TIM
│   ├── loader.rs     # Load YAML schemas from directory → SchemaRegistry
│   ├── validator.rs  # Validate JSON against schema (recursive, constraint-aware)
│   └── template.rs   # Generate template responses from schema
├── intent/           # XML processing
│   ├── parser.rs     # Generic recursive XML → JSON (not type-specific)
│   └── types.rs      # ValidatedIntent (metadata + serde_json::Value payload)
├── dispatch/         # Routing
│   ├── matcher.rs    # Config-driven glob matching
│   └── sender.rs     # HTTP delivery
├── http/             # API surface
│   ├── router.rs     # Routes + AppState
│   └── handlers/     # 7 endpoints
├── config.rs         # YAML config (server, schema dir, dispatchers)
└── lib.rs
```

## Extension Model

| Goal | Action | Code change? |
|------|--------|-------------|
| New intent type | Add `intents/my_type.yaml` | No |
| New field on existing type | Edit the schema YAML | No |
| New validation constraint | Edit the schema YAML | No |
| New chain | Add dispatcher in config | No |
| Swap executor | Change endpoint URL in config | No |
| New XML shorthand | Add to `xml_shorthands` in schema YAML | No |
| New template variant | Add to `template_variants` in schema YAML | No |
| New validation primitive (e.g. regex) | Extend `schema/validator.rs` | Yes — small |
| New dispatch strategy (e.g. fanout) | Extend `dispatch/` | Yes |

## Key Design Decisions

### Schema-driven, not code-driven

Intent types are YAML, not Rust enums. This means:
- Platform teams can add intent types without Rust knowledge
- Schema files are version-controllable and diffable
- The same schema drives validation AND agent templates — no drift

### Generic XML parser

The parser doesn't know about "buy" or "sell". It recursively converts any XML tree into JSON. Type-specific validation is handled by the schema engine.

This means the parser never needs to change when intent types are added.

### ValidatedIntent is just metadata + JSON

```rust
pub struct ValidatedIntent {
    pub intent_id: String,
    pub intent_type: String,       // from schema, not enum
    pub chain_id: String,
    pub payload: serde_json::Value, // schema-validated
    pub raw_xml: String,
    pub received_at: DateTime<Utc>,
}
```

No Rust structs per intent type. The schema YAML defines the structure; executors parse the JSON payload according to their needs.

### Templates are the agent interface

The `/templates` endpoint is how AI agents discover TIM's capabilities. An agent's system prompt only needs:

```
To trade: GET /api/v1/templates to see available actions.
Pick a template, fill in the placeholders, POST to /api/v1/dispatch.
```

No hardcoded XML examples in the agent prompt — they come from TIM at runtime.
