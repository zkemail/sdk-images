## ZKEmail Circom Contracts

This package contains the pre-generated Solidity verifier contracts for a single ZKEmail blueprint.  
These contracts are built by the pipeline whenever a new blueprint is created, then zipped and exposed on the website so that anyone can download and deploy their own verifier.

### What lives in this package

- `src/Groth16Verifier.sol` - Groth16 verifier contract for the blueprint's proving key.
- `src/ZKEmailVerifier.sol` - ZKEmail verifier that plugs into the shared DKIM registry and Groth16 verifier.
- `src/interfaces/IGroth16Verifier.sol` - interface for the Groth16 verifier.
- `src/interfaces/IDKIMRegistry.sol` - interface used to talk to the DKIM registry.
- `src/interfaces/IZKEmailVerifier.sol` - interface for the ZKEmail verifier contract.
- `test/DKIMRegistryMock.sol` - simple mock of the DKIM registry for testing.
- `hh-ignition/modules/ZKEmailVerifier.ts` - Hardhat Ignition module that deploys `Groth16Verifier` and `ZKEmailVerifier`.
- `script/DeployZKEmailVerifier.s.sol` - Foundry deployment script retained for manual/advanced usage.
- `script/verify-zk-email-verifier.sh` - Foundry verification helper for explorers that support standard Etherscan APIs.

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
| `CHAIN_ID`          | Optional                      | Numeric chain ID used by `script/verify-zk-email-verifier.sh` (Foundry helper).           |
| `ETHERSCAN_API_KEY` | For verification only | API key for Etherscan-compatible block explorer (for example Base Sepolia). |

### Deploying with Hardhat Ignition

Deployment is handled through Hardhat Ignition using `hh-ignition/modules/ZKEmailVerifier.ts`.

Install dependencies:

```bash
yarn
```

Build:

```bash
yarn build
```

Deploy (pass a network from `hardhat.config.ts`, e.g. `84532` for Base Sepolia or `420420417` for Polkadot Hub Testnet):

```bash
yarn deploy 84532
```

Verify contracts for the same network:

```bash
yarn verify chain-84532
```

Hardhat Ignition stores deployment artifacts under `hh-ignition/deployments`, and verification uses those artifacts.

### Optional: deploying with Foundry script

For manual/advanced flows you can still use the Foundry script:

```bash
forge script script/DeployZKEmailVerifier.s.sol:DeployZKEmailVerifierScript \
  --rpc-url $RPC_URL \
  --broadcast
```

### All available commands

| Command       | Description                                                                 |
| ------------- | --------------------------------------------------------------------------- |
| `yarn build`  | Compile contracts with Hardhat (`hardhat compile`).                         |
| `yarn deploy` | Deploy with Hardhat Ignition (`hardhat ignition deploy ... --network <id>`). |
| `yarn verify` | Verify Ignition deployments (use `yarn verify chain-<chainid>`). |
