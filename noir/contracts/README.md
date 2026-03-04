## ZKEmail Noir Contracts

This package contains the pre-generated Solidity verifier contracts for a single ZKEmail blueprint.  
These contracts are built by the pipeline whenever a new blueprint is created, then zipped and exposed on the website so that anyone can download and deploy their own verifier.

### What lives in this package

- `src/HonkVerifier.sol` - Honk verifier verifier contract for the blueprint’s proving key.
- `src/ZKEmailVerifier.sol` - ZKEmail verifier that plugs into the shared DKIM registry and Honk verifier.
- `src/interfaces/IHonkVerifier.sol` - interface for the Honk verifier.
- `src/interfaces/IDKIMRegistry.sol` - interface used to talk to the DKIM registry.
- `src/interfaces/IZKEmailVerifier.sol` - interface for the ZKEmail verifier contract.
- `test/DKIMRegistryMock.sol` - simple mock of the DKIM registry for testing.
- `script/DeployZKEmailVerifier.s.sol` - Foundry script that deploys a fresh `HonkVerifier` and `ZKEmailVerifier` against an existing DKIM registry instance.

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

### Deploying with Foundry

Deployment is handled via Foundry’s `forge` CLI. This repository assumes you have Foundry installed globally (via `foundryup`) rather than as an NPM dependency.

**Required environment variables:**

- `PRIVATE_KEY` - EOA private key used by Foundry to broadcast transactions.
- `DKIM_REGISTRY` - address of the already-deployed `DKIMRegistry` contract.
- `RPC_URL` - RPC URL for the target network (passed to the `deploy` script via `--rpc-url`).

The `package.json` scripts are thin wrappers around `forge` and can be run with Yarn.

First install dependencies:

```bash
yarn
```

Then you can:

- Build:

  ```bash
  yarn build
  # equivalent to:
  # forge build
  ```

- Deploy:

  ```bash
  yarn deploy
  # equivalent to:
  # forge script script/DeployZKEmailVerifier.s.sol --broadcast --non-interactive --rpc-url $RPC_URL
  ```
