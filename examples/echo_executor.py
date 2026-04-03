"""
Echo executor — prints dispatched intents and returns a mock response.

    pip install flask
    python examples/echo_executor.py
"""
from flask import Flask, request, jsonify
import json

app = Flask(__name__)

@app.route("/execute", methods=["POST"])
def execute():
    p = request.json
    print(f"\n{'='*50}")
    print(f"Intent: {p.get('intent_id')}")
    print(f"  type:  {p.get('intent_type')}")
    print(f"  chain: {p.get('chain_id')}")
    print(f"  payload: {json.dumps(p.get('payload', {}), indent=2)}")
    print(f"{'='*50}\n")

    return jsonify({
        "status": "confirmed",
        "intent_id": p.get("intent_id"),
        "transaction_hash": "ECHO_" + "0" * 40,
        "details": f"Echo: received {p.get('intent_type')} on {p.get('chain_id')}",
    })

if __name__ == "__main__":
    print("Echo executor on http://localhost:3001")
    app.run(port=3001)
