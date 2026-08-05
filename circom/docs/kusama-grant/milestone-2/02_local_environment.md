# 02 - Local Environment

Milestone 2 local testing environment: Hardhat network profiles (`localPvm`, `localEvm`) and helpers for deploying to a local PolkaVM-compatible node and to Anvil.

## Implementation Notes

- The contracts package includes local network definitions in Hardhat for PolkaVM-compatible execution.
- Local PolkaVM dev binaries can be downloaded via helper script for supported platforms.
- Deployment command entry points exist and can target configured local network profiles.
- `ZKEmailVerifier.sol`, `Groth16Verifier.sol` (mock), and `interfaces/IGroth16Verifier.sol` are **generated** (gitignored). You must materialize them under [`circom/contracts/src/`](../../../contracts/src/) before `yarn build`; the Circom crate provides `generate-example-contracts` for a fixed example payload (see the how-to).

## Local Environment Components

- Local PolkaVM-compatible network configuration:
  - [`circom/contracts/hardhat.config.ts`](../../../contracts/hardhat.config.ts) (`hardhat`, `localPvm`, and `localEvm` network entries)
- PolkaVM local node setup helper:
  - [`circom/contracts/bin/setup-dev-node.sh`](../../../contracts/bin/setup-dev-node.sh)
- Build/deploy command scripts:
  - [`circom/contracts/package.json`](../../../contracts/package.json)
- Example contract payload + generator CLI:
  - [`circom/example-contract-data.json`](../../../example-contract-data.json)
  - `cargo run -p circom -- generate-example-contracts …` (documented in [`circom/README.md`](../../../README.md))

## Expected Local Flow

- Generate example Solidity contracts into [`circom/contracts/src/`](../../../contracts/src/) (required; not committed in git).
- Prepare local node binaries (where needed).
- Start local execution target.
- Build contracts (`yarn build` from [`circom/contracts`](../../../contracts/)).
- Deploy contracts to local target network.

## Runnable Demonstration

The full copy-pasteable command sequence for both local targets (generate contracts, set up the PolkaVM node or Anvil, build, and deploy) with representative outputs is in the public how-to: [`05_public_howto.md`](./05_public_howto.md) (steps 2 through 5).
