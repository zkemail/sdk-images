#!/bin/bash
set -euo pipefail

if [ -f .env ]; then
  set -a
  source .env
  set +a
fi

: "${ETHERSCAN_API_KEY:?ETHERSCAN_API_KEY is required}"
: "${CHAIN_ID:?CHAIN_ID is required}"
: "${DKIM_REGISTRY:?DKIM_REGISTRY is required}"

BROADCAST_FILE="broadcast/DeployZKEmailVerifier.s.sol/${CHAIN_ID}/run-latest.json"

if [ -z "${GROTH16_VERIFIER:-}" ] || [ -z "${ZK_EMAIL_VERIFIER:-}" ]; then
  if [ ! -f "$BROADCAST_FILE" ]; then
    echo "Error: GROTH16_VERIFIER and ZK_EMAIL_VERIFIER not set and broadcast file not found at $BROADCAST_FILE"
    echo "Either set them in .env or run the deploy script first."
    exit 1
  fi
  echo "Reading deployed addresses from $BROADCAST_FILE"
  GROTH16_VERIFIER="${GROTH16_VERIFIER:-$(jq -r '.transactions[] | select(.contractName == "Groth16Verifier") | .contractAddress' "$BROADCAST_FILE")}"
  ZK_EMAIL_VERIFIER="${ZK_EMAIL_VERIFIER:-$(jq -r '.transactions[] | select(.contractName == "ZKEmailVerifier") | .contractAddress' "$BROADCAST_FILE")}"
fi

if [ -z "$GROTH16_VERIFIER" ] || [ -z "$ZK_EMAIL_VERIFIER" ]; then
  echo "Error: Could not determine GROTH16_VERIFIER or ZK_EMAIL_VERIFIER addresses"
  exit 1
fi

RETRIES="${RETRIES:-5}"
DELAY="${DELAY:-10}"

CONSTRUCTOR_ARGS=$(cast abi-encode "constructor(address,address)" "$DKIM_REGISTRY" "$GROTH16_VERIFIER")

echo "=== Verifying Groth16Verifier at $GROTH16_VERIFIER ==="
forge verify-contract \
  --chain-id "$CHAIN_ID" \
  --etherscan-api-key "$ETHERSCAN_API_KEY" \
  --watch \
  --retries "$RETRIES" \
  --delay "$DELAY" \
  "$GROTH16_VERIFIER" \
  src/Groth16Verifier.sol:Groth16Verifier

echo ""

echo "=== Verifying ZKEmailVerifier at $ZK_EMAIL_VERIFIER ==="
forge verify-contract \
  --chain-id "$CHAIN_ID" \
  --etherscan-api-key "$ETHERSCAN_API_KEY" \
  --watch \
  --retries "$RETRIES" \
  --delay "$DELAY" \
  --constructor-args "$CONSTRUCTOR_ARGS" \
  "$ZK_EMAIL_VERIFIER" \
  src/ZKEmailVerifier.sol:ZKEmailVerifier

echo ""
echo "=== All contracts verified ==="
