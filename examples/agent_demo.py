"""
Agent demo — shows how an AI agent discovers and uses TIM's intent gateway.

Prerequisites:
    1. Start TIM:          cargo run --bin tim_server
    2. Start echo executor: pip install flask && python examples/echo_executor.py

Usage:
    pip install requests
    python examples/agent_demo.py
"""
import re
import sys
import requests

TIM_BASE = "http://localhost:8080/api/v1"


def discover_templates():
    """Step 1: Agent discovers what intent types are available."""
    print("=== Step 1: Discover available intent types ===\n")
    resp = requests.get(f"{TIM_BASE}/templates")
    resp.raise_for_status()
    templates = resp.json()
    for t in templates:
        print(f"  {t['name']}: {t['description']}")
        if t.get("variants"):
            print(f"    variants: {', '.join(t['variants'])}")
    print()
    return templates


def fetch_template(intent_type: str) -> dict:
    """Step 2: Agent fetches the XML template and field docs for a specific type."""
    print(f"=== Step 2: Fetch template for {intent_type} ===\n")
    resp = requests.get(f"{TIM_BASE}/templates/{intent_type}")
    resp.raise_for_status()
    info = resp.json()
    print(f"  Default template:\n{info.get('template', '(none)')}")
    return info


def fill_template(xml_template: str, values: dict) -> str:
    """Step 3: Agent fills in the template placeholders."""
    xml = xml_template
    for key, val in values.items():
        xml = xml.replace("{{" + key + "}}", str(val))
    # Check for unfilled placeholders
    remaining = re.findall(r"\{\{(\w+)\}\}", xml)
    if remaining:
        print(f"  Warning: unfilled placeholders: {remaining}")
    return xml


def validate_intent(xml: str) -> dict:
    """Step 4: Agent validates the intent before dispatching."""
    print("=== Step 4: Validate intent ===\n")
    resp = requests.post(f"{TIM_BASE}/validate", json={"intent": xml})
    if resp.status_code != 200:
        print(f"  Validation FAILED: {resp.json()}")
        return None
    result = resp.json()
    print(f"  Validation OK — intent_id={result['intent_id']}, type={result['intent_type']}")
    print()
    return result


def dispatch_intent(xml: str) -> dict:
    """Step 5: Agent dispatches the intent for execution."""
    print("=== Step 5: Dispatch intent ===\n")
    resp = requests.post(f"{TIM_BASE}/dispatch", json={"intent": xml})
    if resp.status_code != 200:
        print(f"  Dispatch FAILED ({resp.status_code}): {resp.json()}")
        return None
    result = resp.json()
    print(f"  Dispatch OK: {result}")
    print()
    return result


def demo_immediate():
    """Full workflow: discover → fetch → fill → validate → dispatch an IMMEDIATE intent."""
    print("\n" + "=" * 60)
    print("  DEMO: IMMEDIATE swap (buy SOL tokens)")
    print("=" * 60 + "\n")

    # Fetch template
    info = fetch_template("IMMEDIATE")

    # Pick the 'buy' variant
    buy_variant = next(
        (v for v in info.get("variants", []) if v["name"] == "buy"), None
    )
    if not buy_variant:
        print("  No 'buy' variant found, using default template")
        xml_template = info["template"]
    else:
        xml_template = buy_variant["xml"]
        print(f"  Using variant: {buy_variant['name']} — {buy_variant['description']}\n")

    # Fill template
    print("=== Step 3: Fill template ===\n")
    xml = fill_template(xml_template, {
        "chain_id": "solana:mainnet-beta",
        "amount": "0.5",
        "quote_token_address": "So11111111111111111111111111111111111111112",
        "base_token_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    })
    print(f"  Filled XML:\n{xml}")

    # Validate
    if not validate_intent(xml):
        return

    # Dispatch
    dispatch_intent(xml)


def demo_limit_order():
    """Full workflow for a LIMIT_ORDER intent."""
    print("\n" + "=" * 60)
    print("  DEMO: LIMIT_ORDER (buy when price drops)")
    print("=" * 60 + "\n")

    # Fetch template
    info = fetch_template("LIMIT_ORDER")

    # Pick the 'buy_below' variant
    variant = next(
        (v for v in info.get("variants", []) if v["name"] == "buy_below"), None
    )
    if not variant:
        print("  No 'buy_below' variant found, using default template")
        xml_template = info["template"]
    else:
        xml_template = variant["xml"]
        print(f"  Using variant: {variant['name']} — {variant['description']}\n")

    # Fill template
    print("=== Step 3: Fill template ===\n")
    xml = fill_template(xml_template, {
        "chain_id": "solana:mainnet-beta",
        "limit_price": "120.50",
        "amount": "1.0",
        "quote_token_address": "So11111111111111111111111111111111111111112",
        "base_token_address": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "max_slippage_percent": "2.0",
    })
    print(f"  Filled XML:\n{xml}")

    # Validate
    if not validate_intent(xml):
        return

    # Dispatch
    dispatch_intent(xml)


def main():
    print("TIM Agent Demo")
    print("Connecting to TIM at", TIM_BASE)
    print()

    # Check TIM is running
    try:
        requests.get("http://localhost:8080/health", timeout=2)
    except requests.ConnectionError:
        print("ERROR: Cannot connect to TIM. Start it first:")
        print("  cargo run --bin tim_server")
        sys.exit(1)

    # Step 1: Discover
    discover_templates()

    # Run demos
    demo_immediate()
    demo_limit_order()

    print("\nDone. All intents dispatched successfully.")


if __name__ == "__main__":
    main()
