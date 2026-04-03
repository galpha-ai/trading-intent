# Intent XML Schema

## Format

All intents share the same root structure:

```xml
<intent>
  <type>INTENT_TYPE</type>
  <chain_id>CAIP-2 chain ID</chain_id>
  <!-- type-specific fields -->
</intent>
```

The available intent types and their fields are defined by YAML schemas in `intents/`.

## Built-in Types

### IMMEDIATE — Swap tokens on-chain

**Buy:**
```xml
<intent>
  <type>IMMEDIATE</type>
  <chain_id>solana:mainnet-beta</chain_id>
  <entry>
    <condition><immediate>true</immediate></condition>
    <action>
      <buy>
        <amount>0.1</amount>
        <quote>So11111111111111111111111111111111111111112</quote>
        <base>EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v</base>
      </buy>
    </action>
  </entry>
</intent>
```

**Sell (percentage):**
```xml
<action>
  <sell>
    <relative><percentage>50.0</percentage></relative>
    <quote>So11111111111111111111111111111111111111112</quote>
    <base>EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v</base>
  </sell>
</action>
```

**Sell all (shorthand):**
```xml
<amount>all</amount>
<!-- Expands to: <relative><percentage>100.0</percentage></relative> -->
```

**With exit strategy:**
```xml
<exit>
  <conditions>
    <profit_percent>50</profit_percent>
    <loss_percent>30</loss_percent>
  </conditions>
  <logic>OR</logic>
</exit>
```

### CONDITIONAL_ENTRY — Event-triggered rule

```xml
<intent>
  <type>CONDITIONAL_ENTRY</type>
  <chain_id>solana:mainnet-beta</chain_id>
  <entry>
    <condition>
      <event_trigger>
        <event_type>news_signal</event_type>
        <platform>x</platform>
        <author_handle>example</author_handle>
        <criteria>Buy when bullish signal detected</criteria>
      </event_trigger>
    </condition>
    <action>
      <buy>
        <amount>10</amount>
        <quote>QUOTE_ADDR</quote>
        <base>BASE_ADDR</base>
      </buy>
    </action>
  </entry>
</intent>
```

## Adding Custom Types

Create `intents/my_type.yaml` with the schema definition. See existing schemas for format. The type will automatically appear in `/api/v1/templates` and validate in `/api/v1/dispatch`.

## Parser Behavior

- **Case-insensitive**: `<TYPE>`, `<Type>`, `<type>` are equivalent
- **Whitespace tolerant**: leading/trailing whitespace trimmed
- **Unknown elements preserved**: extra elements pass through to JSON (schema decides whether to reject them)
- **Shorthands applied before parsing**: configured per schema

## Chain IDs

[CAIP-2](https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md) format:

| Chain | ID |
|-------|----|
| Solana Mainnet | `solana:mainnet-beta` |
| Ethereum | `eip155:1` |
| Base | `eip155:8453` |
| Arbitrum | `eip155:42161` |
