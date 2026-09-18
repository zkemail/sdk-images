## ZKEmail Noir Contracts

This package contains the pre-generated Solidity verifier contracts for a single ZKEmail blueprint.  
These contracts are built by the pipeline whenever a new blueprint is created, then zipped and exposed on the website so that anyone can download and deploy their own verifier.

### What lives in this package

- `src/HonkVerifier.sol` - Honk verifier contract for the blueprint's proving key.
- `src/ZKEmailVerifier.sol` - ZKEmail verifier that plugs into the shared DKIM registry and Honk verifier.
- `src/interfaces/IHonkVerifier.sol` - interface for the Honk verifier.
- `src/interfaces/IDKIMRegistry.sol` - interface used to talk to the DKIM registry.
- `src/interfaces/IZKEmailVerifier.sol` - interface for the ZKEmail verifier contract.
- `test/DKIMRegistryMock.sol` - simple mock of the DKIM registry for testing.
- `script/DeployZKEmailVerifier.s.sol` - Foundry script that deploys `HonkVerifier` and `ZKEmailVerifier` against an existing DKIM registry instance.
- `script/verify-zk-email-verifier.sh` - Shell script to verify both contracts on Etherscan-compatible explorers via Foundry.
- `hh-scripts/deploy-zk-email-verifier.ts` - Hardhat script for deploying to Polkadot Hub.
- `hh-scripts/verify-zk-email-verifier.ts` - Hardhat script for verifying contracts on Polkadot Hub (Blockscout).

### DKIM registry code / repo

These contracts **do not** include the DKIM registry implementation itself.  
They expect an existing contract that implements `IDKIMRegistry` to already be deployed.

The `IDKIMRegistry` interface in this package is adapted from the implementation in `@zk-email/zk-email-verify`, see: [`zk-email-verify#abe9d839d2518ef3f3f0a5fab2283fd672672dde`](https://github.com/zkemail/zk-email-verify/tree/abe9d839d2518ef3f3f0a5fab2283fd672672dde/packages/contracts).

When deploying, you typically have two options:

1. **Use an existing deployed DKIM registry (recommended)** - for example the `UserOverrideableDKIMRegistry` instances documented in  
   the Account Recovery docs: [`docs.zk.email/account-recovery/deployed-contracts`](https://docs.zk.email/account-recovery/deployed-contracts).  
   In this case you simply configure `DKIM_REGISTRY` to point at one of those addresses.
2. **Deploy your own DKIM registry** - for example using the contracts from `@zk-email/contracts`:

   ```solidity
   import { DKIMRegistry } from "@zk-email/contracts/DKIMRegistry.sol";

   DKIMRegistry registry = new DKIMRegistry(owner);
   ```

> Whichever option you choose, make sure the registry you point to implements the **ERC-7969 DKIM registry interface** (`IERC7969` / `IDKIMRegistry`) as defined in [`IERC7969.sol` in zkemail/zk-email-verify](https://github.com/zkemail/zk-email-verify/blob/abe9d839d2518ef3f3f0a5fab2283fd672672dde/packages/contracts/interfaces/IERC7969.sol).

Once you have a `DKIMRegistry` address (from either 1 or 2), pass that address into the verifier deployment flow (see below).

### Environment variables

Copy `.env.example` to `.env` and fill in the values:

| Variable            | Required                      | Description                                                                                |
| ------------------- | ----------------------------- | ------------------------------------------------------------------------------------------ |
| `PRIVATE_KEY`       | Yes                           | EOA private key used to broadcast transactions.                                            |
| `DKIM_REGISTRY`     | Yes                           | Address of the already-deployed `DKIMRegistry` contract.                                   |
| `RPC_URL`           | Yes                           | RPC URL for the target network.                                                            |
| `CHAIN_ID`          | Yes                           | Numeric chain ID (used by verification scripts).                                           |
| `ETHERSCAN_API_KEY` | For Foundry verification only | API key for Etherscan-compatible block explorer. Not needed for Polkadot Hub (Blockscout). |
| `HONK_VERIFIER`     | Optional                      | Deployed `HonkVerifier` address (auto-read from deployment files if omitted).              |
| `ZK_EMAIL_VERIFIER` | Optional                      | Deployed `ZKEmailVerifier` address (auto-read from deployment files if omitted).           |

### Deploying with Foundry (EVM chains)

Deployment is handled via Foundry's `forge` CLI. This repository assumes you have Foundry installed globally (via `foundryup`) rather than as an NPM dependency.

Install dependencies:

```bash
yarn
```

Build:

```bash
yarn build
```

Deploy:

```bash
yarn deploy
```

Verify on Etherscan (after deploying):

```bash
yarn verify
```

The verification script reads deployed addresses from Foundry's `broadcast/DeployZKEmailVerifier.s.sol/<CHAIN_ID>/run-latest.json`. You can also set `HONK_VERIFIER` and `ZK_EMAIL_VERIFIER` explicitly in `.env` to skip the auto-detection.

### Deploying to Polkadot Hub

Polkadot Hub deployment uses Hardhat with the `@parity/hardhat-polkadot` plugin.

#### Polkadot Hub Testnet

Build (compiles with `resolc` for PolkaVM):

```bash
yarn build:polka
```

Deploy to Polkadot Hub Testnet:

```bash
yarn deploy:polka
```

Verify on Blockscout (after deploying):

```bash
yarn verify:polka
```

The Polkadot deploy script saves addresses to `hh-deployments/<chainId>/run-latest.json`. The verify script reads from there automatically, or you can override with `HONK_VERIFIER` / `ZK_EMAIL_VERIFIER` env vars.

### All available commands

| Command             | Description                                                          |
| ------------------- | -------------------------------------------------------------------- |
| `yarn build`        | Compile contracts with Foundry (`forge build`).                      |
| `yarn build:polka`  | Compile contracts with Hardhat + `resolc` for Polkadot Hub.          |
| `yarn deploy`       | Deploy via Foundry to the chain at `$RPC_URL`.                       |
| `yarn deploy:polka` | Deploy via Hardhat to Polkadot Hub Testnet.                          |
| `yarn verify`       | Verify both contracts on an Etherscan-compatible explorer (Foundry). |
| `yarn verify:polka` | Verify both contracts on Polkadot Hub Blockscout (Hardhat).          |
