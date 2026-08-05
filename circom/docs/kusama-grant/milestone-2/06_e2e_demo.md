# 06 - End-to-End Paseo Demo

Real wrapper, real Groth16 verifier, real proof, and the real milestone-1 `DKIMRegistry` -- all
live on Paseo, added in response to review feedback (not itself a milestone deliverable; see
[`README.md`](./README.md) for the deliverables this milestone actually commits to).

## What this demonstrates

The committed `testBlueprint` fixture (see
[`test/fixtures/testBlueprint/README.md`](../../../contracts/test/fixtures/testBlueprint/README.md))
is normally exercised in Foundry against a mock registry, since the wrapper's own logic (proof
decoding, offset math, the DKIM-gate call) is independent of which concrete `IDKIMRegistry` it's
wired to. This demo removes that abstraction entirely: the exact fixture contracts are deployed to
Paseo and wired to the real, already-live milestone-1 `DKIMRegistry`
(`0xD9e492f8104Ec730AF47A1A5C0cEAf94C89Da8EE`), not a fresh registry deployed for the occasion.

## Deployment Manifest

| Field | Value |
| --- | --- |
| `TestBlueprintGroth16Verifier` | [`0x2B1D8681B837a9e9080D36E9b588D1275D8e5D04`](https://blockscout-testnet.polkadot.io/address/0x2B1D8681B837a9e9080D36E9b588D1275D8e5D04) |
| `TestBlueprintZKEmailVerifier` | [`0x68E81c9909aD3982A991a953A63729bbF906B72B`](https://blockscout-testnet.polkadot.io/address/0x68E81c9909aD3982A991a953A63729bbF906B72B) |
| `DKIM_REGISTRY` (constructor arg) | `0xD9e492f8104Ec730AF47A1A5C0cEAf94C89Da8EE` -- the real milestone-1 `DKIMRegistry`, confirmed by reading it back on-chain (see Reproduce below), not a fresh registry |
| `DOMAIN_HASH` (wrapper constant) | `0xbbcc9f0af825b951a41a390086b09f7d8b4c4434d5315255b2ab6ffee1e8c781` (`keccak256("x.com")`), confirmed by reading it back on-chain |
| Network | Polkadot Hub Testnet (Paseo Assethub), chain ID `420420417` |
| `resolc` version | `0.5.0+commit.046455.llvm-18.1.8` |
| Source | [`circom/contracts/src/e2e-demo/`](../../../contracts/src/e2e-demo/) -- verbatim copy of the committed `test/fixtures/testBlueprint/` contracts (only relative import paths differ; no logic differs, confirmed by diff) |
| Ignition module | [`hh-ignition/modules/E2EDemo.ts`](../../../contracts/hh-ignition/modules/E2EDemo.ts) |

## Bytecode Provenance

Same method as milestone 1: compare the locally `resolc`-compiled runtime bytecode hash against
the on-chain code, and confirm the PolkaVM magic-byte prefix.

| Contract | keccak256(deployedBytecode) | Match | PVM magic |
| --- | --- | --- | --- |
| `TestBlueprintGroth16Verifier` | `0xc1d5f187d0d06a9a5a31f2254299fd0f3190efe19086e7192b3c3af430f586ee` | on-chain == local | `0x50564d0000` |
| `TestBlueprintZKEmailVerifier` | `0x5655a9fe87e1a1bd736b117c89a181c71f7985a682f419e3396574165a37ef55` | on-chain == local | `0x50564d0000` |

## Real Proof, Real Registry, Real Transaction

`x.com`'s DKIM key hash (the exact value the fixture's real proof was built against -- see
[`test/fixtures/testBlueprint/README.md`](../../../contracts/test/fixtures/testBlueprint/README.md)
for provenance of the proof itself) is registered in the real milestone-1 registry. `verify()` was
called with the fixture's real proof and public inputs, submitted as an actual mined transaction
(not just a read-only call) so it's independently inspectable without any tooling:

- Tx: [`0x93599793d97386319a1a0da54807b9bf1c2e387bbaa9a2e267eaa266f1bfb23c`](https://blockscout-testnet.polkadot.io/tx/0x93599793d97386319a1a0da54807b9bf1c2e387bbaa9a2e267eaa266f1bfb23c)
- Block: `11482497`
- Status: success, `gasUsed: 5961`

**Negative control:** flipping a single bit of the DKIM key hash in the public inputs (`publicInputs[0]`, the value gated by `DKIM_REGISTRY.isKeyHashValid`) reverts with `InvalidPublicKey()` (selector `0xa2d0fee8`), confirming the call genuinely exercises the real registry's check rather than passing through unconditionally.

## Reproduce

From `circom/contracts`:

```bash
# confirm the registry wired into the deployed wrapper is the real milestone-1 one
cast call 0x68E81c9909aD3982A991a953A63729bbF906B72B "DKIM_REGISTRY()(address)" \
  --rpc-url https://eth-rpc-testnet.polkadot.io

# confirm the domain hash
cast call 0x68E81c9909aD3982A991a953A63729bbF906B72B "DOMAIN_HASH()(bytes32)" \
  --rpc-url https://eth-rpc-testnet.polkadot.io

# PVM bytecode magic
cast code 0x68E81c9909aD3982A991a953A63729bbF906B72B \
  --rpc-url https://eth-rpc-testnet.polkadot.io | cut -c1-12

# on-chain vs local runtime-bytecode hash
cast code 0x68E81c9909aD3982A991a953A63729bbF906B72B \
  --rpc-url https://eth-rpc-testnet.polkadot.io | cast keccak
jq -r '.deployedBytecode' hh-artifacts/src/e2e-demo/TestBlueprintZKEmailVerifier.sol/TestBlueprintZKEmailVerifier.json | cast keccak
```

## Source-of-Truth Policy

Same as milestone 1: this doc and the deployment manifest above are the canonical evidence.
`circom/contracts/hh-ignition/deployments/chain-420420417/` is committed for reproducibility
(real-network deployments only; local-only `chain-31337`/`chain-420420420` stay gitignored).
