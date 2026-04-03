# Dispatch Protocol

## Request

TIM POSTs `ValidatedIntent` JSON to the matched executor endpoint.

### Headers

| Header | Value |
|--------|-------|
| `Content-Type` | `application/json` |
| `X-TIM-Intent-ID` | UUID assigned by TIM |
| `X-TIM-Intent-Type` | e.g. `IMMEDIATE` |
| `X-TIM-Chain-ID` | e.g. `solana:mainnet-beta` |

Plus any custom `headers` from dispatcher config.

### Body

```json
{
  "intent_id": "550e8400-...",
  "intent_type": "IMMEDIATE",
  "chain_id": "solana:mainnet-beta",
  "payload": {
    "type": "IMMEDIATE",
    "chain_id": "solana:mainnet-beta",
    "entry": {
      "condition": { "immediate": true },
      "action": {
        "buy": {
          "amount": 0.1,
          "quote": "So11111111111111111111111111111111111111112",
          "base": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        }
      }
    }
  },
  "raw_xml": "<intent>...</intent>",
  "received_at": "2024-01-01T00:00:00Z"
}
```

`payload` is the full validated JSON tree from the XML. Its structure matches the intent schema YAML.

## Response

### Success (2xx)

Return any JSON. TIM forwards it as-is.

```json
{
  "status": "confirmed",
  "transaction_hash": "5UB3...",
  "details": "Bought 1234 USDC"
}
```

### Error (4xx/5xx)

TIM forwards the status code and body to the caller.

## Timeout

Configurable per dispatcher. Default 30s. Returns `504 Gateway Timeout`.

## Idempotency

Use `intent_id` (UUID) for deduplication.
