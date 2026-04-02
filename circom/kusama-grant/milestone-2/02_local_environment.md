# 02 - Local Environment

Deliverable mapping: Milestone 2, Deliverable 2 (`Local Environment`).

## What must be delivered

- Local testing environments for both EVM and PolkaVM.
- Ability to deploy/test contracts on local nodes.

## Implementation Notes

- The contracts package includes local network definitions in Hardhat for PolkaVM-compatible execution.
- Local PolkaVM dev binaries can be downloaded via helper script for supported platforms.
- Deployment command entry points exist and can target configured local network profiles.
- `ZKEmailVerifier.sol`, `Groth16Verifier.sol` (mock), and `interfaces/IGroth16Verifier.sol` are **generated** (gitignored). You must materialize them under `circom/contracts/src/` before `yarn build`; the Circom crate provides `generate-example-contracts` for a fixed example payload (see below).

## Local Environment Components

- Local PolkaVM-compatible network configuration:
  - `circom/contracts/hardhat.config.ts` (`hardhat`, `localPvm`, and `localEvm` network entries)
- PolkaVM local node setup helper:
  - `circom/contracts/bin/setup-dev-node.sh`
- Build/deploy command scripts:
  - `circom/contracts/package.json`
- Example contract payload + generator CLI:
  - [`circom/example-contract-data.json`](../../example-contract-data.json)
  - `cargo run -p circom -- generate-example-contracts …` (documented in [`circom/README.md`](../../README.md))

## Expected Local Flow

- Generate example Solidity contracts into `circom/contracts/src/` (required; not committed in git).
- Prepare local node binaries (where needed).
- Start local execution target.
- Build contracts (`yarn build` from `circom/contracts`).
- Deploy contracts to local target network.

## Proof-by-Demonstration Commands

Use two directories: run **contract generation** from **`circom/`** (so `./templates/` resolves), then run **install / build / deploy** from **`circom/contracts/`** unless noted.

### 0) Generate example contracts (do this first)

From the **`circom`** directory:

```bash
cargo run -p circom -- generate-example-contracts ./example-contract-data.json ./contracts/src
```

This writes (among others):

- `contracts/src/ZKEmailVerifier.sol`
- `contracts/src/Groth16Verifier.sol` (mock Groth16 verifier from the Tera template)
- `contracts/src/interfaces/IGroth16Verifier.sol`

Example command output:

```text
Populated ZKEmailVerifier contract written to contracts/src/ZKEmailVerifier.sol
Populated MockGroth16Verifier contract written to contracts/src/Groth16Verifier.sol
```

See [`circom/README.md`](../../README.md) for the `ContractData` JSON shape if you need a custom payload.

First-time Hardhat dependency install (once per machine), from the `circom/contracts` directory:

```bash
yarn
```

The flows **A)** and **B)** below assume **0)** is already done.

### A) Local PolkaVM-compatible node flow

Run node setup and Hardhat from **`circom/contracts`** (from repository root: `cd circom/contracts`).

terminal 1:

```bash
./bin/setup-dev-node.sh
```

output:

```text
Downloading dev-node from https://github.com/paritytech/hardhat-polkadot/releases/download/nodes-19907546951/revive-dev-node-darwin-arm64
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
  0     0    0     0    0     0      0      0 --:--:-- --:--:-- --:--:--     0
100  253M  100  253M    0     0  10.1M      0  0:00:25  0:00:25 --:--:-- 10.0M
Downloading eth-rpc from https://github.com/paritytech/hardhat-polkadot/releases/download/nodes-19907546951/eth-rpc-darwin-arm64
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
  0     0    0     0    0     0      0      0 --:--:-- --:--:-- --:--:--     0
100 64.5M  100 64.5M    0     0  13.5M      0  0:00:04  0:00:04 --:--:-- 16.2M
```

start the local node:

```bash
npx hardhat node
```

output:

```text
Starting server at 127.0.0.1:8000
Running command: ./bin/dev-node --rpc-port=8000 --pruning=archive --dev
Starting the Eth RPC Adapter at 127.0.0.1:8545
Running command: ./bin/eth-rpc --node-rpc-url=ws://localhost:8000 --dev
2026-04-02 14:51:38 Running in --dev mode, RPC CORS has been disabled.
2026-04-02 14:51:38 Running in --dev mode, RPC CORS has been disabled.
2026-04-02 14:51:38 🌐 Connecting to node at: ws://localhost:8000 ...
2026-04-02 14:51:38 🌟 Connected to node at: ws://localhost:8000
2026-04-02 14:51:38 💾 Using in-memory database, keeping only 256 blocks in memory
2026-04-02 14:51:38 〽️ Prometheus exporter started at 127.0.0.1:9616
2026-04-02 14:51:38 Running JSON-RPC server: addr=127.0.0.1:8545,[::1]:8545
2026-04-02 14:51:38 🔌 Subscribing to new blocks (BestBlocks)
2026-04-02 14:51:38 🔌 Subscribing to new blocks (FinalizedBlocks)
2026-04-02 14:51:45.401 INFO main sc_rpc_server: Running JSON-RPC server: addr=127.0.0.1:61021,[::1]:61022
```

populate the `.env` with the test values:

```env
# placeholder for deploy testing (non-zero EOA, not a real DKIM registry; replace to exercise verify)
DKIM_REGISTRY=0x70997970C51812dc3A010C7d01b50e0d17dc79C8
```

terminal 2:

compile the contracts:

```bash
yarn build
```

output:

```text
yarn run v1.22.22
$ hardhat compile
Compiling 5 Solidity files
Successfully compiled 5 Solidity files
✨  Done in 4.24s.
```

deploy using the `localPvm` network:

```
yarn deploy localPvm
```

output:

```text
yarn run v1.22.22
$ yes | hardhat ignition deploy hh-ignition/modules/ZKEmailVerifier.ts --network localPvm
✔ Confirm deploy to network localPvm (420420420)? … yes
Hardhat Ignition 🚀

Deploying [ ZKEmailVerifierModule ]

Batch #1
  Executed ZKEmailVerifierModule#Groth16Verifier

Batch #2
  Executed ZKEmailVerifierModule#ZKEmailVerifier

[ ZKEmailVerifierModule ] successfully deployed 🚀

Deployed Addresses

ZKEmailVerifierModule#Groth16Verifier - 0x3ed62137c5DB927cb137c26455969116BF0c23Cb
ZKEmailVerifierModule#ZKEmailVerifier - 0x962c0940d72E7Db6c9a5F81f1cA87D8DB2B82A23
✨  Done in 1.94s.
```

### B) Local EVM flow (Anvil)

Same prerequisite as **A)**: example contracts generated under `circom/contracts/src/`, and `yarn` run once from `circom/contracts` if needed.

terminal 1:

```bash
anvil
```

output:

```text


                             _   _
                            (_) | |
      __ _   _ __   __   __  _  | |
     / _` | | '_ \  \ \ / / | | | |
    | (_| | | | | |  \ V /  | | | |
     \__,_| |_| |_|   \_/   |_| |_|

    1.5.1-stable (b0a9dd9ced 2025-12-22T11:41:09.812070000Z)
    https://github.com/foundry-rs/foundry

...
...

Chain ID
==================

31337

Base Fee
==================

1000000000

Gas Limit
==================

30000000

Genesis Timestamp
==================

1775136197

Genesis Number
==================

0

Listening on 127.0.0.1:8545
```

populate the `.env` with the test values:

```env
# Anvil default account #0 — public test key; never use on mainnet
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
# placeholder for deploy testing (not a real DKIM registry; replace to exercise verify)
DKIM_REGISTRY=0x70997970C51812dc3A010C7d01b50e0d17dc79C8
```

terminal 2:

```bash
yarn build
```

output:

```text
yarn run v1.22.22
$ hardhat compile
Compiling 5 Solidity files
Successfully compiled 5 Solidity files
✨  Done in 2.04s.
```

deploy using the `localEvm` network (Anvil on `127.0.0.1:8545`):

```bash
yarn deploy localEvm
```

output:

```text
yarn run v1.22.22
$ yes | hardhat ignition deploy hh-ignition/modules/ZKEmailVerifier.ts --network localEvm
Hardhat Ignition 🚀

Deploying [ ZKEmailVerifierModule ]

Batch #1
  Executed ZKEmailVerifierModule#Groth16Verifier

Batch #2
  Executed ZKEmailVerifierModule#ZKEmailVerifier

[ ZKEmailVerifierModule ] successfully deployed 🚀

Deployed Addresses

ZKEmailVerifierModule#Groth16Verifier - 0x5FbDB2315678afecb367f032d93F642f64180aa3
ZKEmailVerifierModule#ZKEmailVerifier - 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
✨  Done in 2.26s.
```

## Evidence Standard for This Deliverable

- For local environment acceptance, this document uses:
  - runnable command sequences for both local targets
  - a prerequisite step that generates gitignored Solidity from templates (`generate-example-contracts`)
  - representative output snippets that confirm successful compile/deploy path

## Status

`Delivered`

Conclusion: The deliverable "Working local setup capable of deploying contracts to Anvil and a local PolkaVM compatible node." is delivered: `hardhat.config.ts` defines `localEvm` (e.g. Anvil on `8545`) and `localPvm` (adapter RPC); this document records generating example contracts, then end-to-end setup, compile, and Ignition deploy commands with representative outputs for both paths.
