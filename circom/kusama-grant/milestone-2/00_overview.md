# Milestone 2 - ZK Verifier Contract Tooling

Primary Goal: Develop a reusable smart contract project with tooling supporting both EVM and PolkaVM deployments, including wrapper contract templates that integrate DKIM registry verification.

## Deliverables

| # | Name | Description | Deliverable | What was done / Proof |
| --- | --- | --- | --- | --- |
| 1 | Project Setup | Create a unified project structure using Hardhat and Foundry. Configure support for both the native Solidity compiler (`solc`) and the PolkaVM Solidity compiler (`resolc`). | Project structure with Hardhat and Foundry configuration supporting EVM and PolkaVM compilation. | Unified Hardhat (primary build/deploy) + Foundry (tests) package with `solc` and `resolc`/`@parity/hardhat-polkadot` build targets, configured in [`hardhat.config.ts`](../../contracts/hardhat.config.ts) and [`foundry.toml`](../../contracts/foundry.toml), with the Ignition module [`ZKEmailVerifier.ts`](../../contracts/hh-ignition/modules/ZKEmailVerifier.ts). **Proof:** [`01_project_setup.md`](./01_project_setup.md). |
| 2 | Local Environment | Configure local testing environments for both EVM and PolkaVM, enabling contract deployment and testing on local nodes. | Working local setup capable of deploying contracts to Anvil and a local PolkaVM compatible node. | `localEvm` (Anvil) and `localPvm` (PolkaVM adapter RPC) networks defined in [`hardhat.config.ts`](../../contracts/hardhat.config.ts); local node helper [`setup-dev-node.sh`](../../contracts/bin/setup-dev-node.sh); reproducible generate → `yarn build` → Ignition deploy commands with captured outputs for both paths. **Proof:** [`02_local_environment.md`](./02_local_environment.md). |
| 3 | Verifier Interface and Wrappers | Define a generic verifier interface and implement wrapper contracts for Groth16 that integrate DKIM verification and prepare/format proofs before invoking the underlying verifier contract. | Verifier interface and wrapper contracts compatible with different proof systems. | Committed integrator interfaces [`IZKEmailVerifier`](../../contracts/src/interfaces/IZKEmailVerifier.sol) and [`IDKIMRegistry`](../../contracts/src/interfaces/IDKIMRegistry.sol); Groth16 + DKIM wrapper emitted per blueprint from [`ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera) via [`contract.rs`](../../src/contract.rs), checking public-input count, validating the DKIM key hash, then formatting/decoding the proof before calling `verifyProof`. **Proof:** [`03_verifier_interface_and_wrappers.md`](./03_verifier_interface_and_wrappers.md). |
| 4 | Template and Tooling | Convert wrapper contracts into reusable templates and provide deployment and verification tooling for both environments. | Templated wrapper contracts, deployment scripts, and verification tooling. | Reusable Tera templates ([`ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera), [`IGroth16Verifier.sol.tera`](../../templates/IGroth16Verifier.sol.tera), [`MockGroth16Verifier.sol.tera`](../../templates/MockGroth16Verifier.sol.tera)) rendered by [`contract.rs`](../../src/contract.rs); deployment via Ignition [`ZKEmailVerifier.ts`](../../contracts/hh-ignition/modules/ZKEmailVerifier.ts) (`yarn deploy <network>`) and verification via [`verify-zk-email-verifier.sh`](../../contracts/script/verify-zk-email-verifier.sh) (`yarn verify`), with PolkaVM explorer caveats noted. **Proof:** [`04_template_and_tooling.md`](./04_template_and_tooling.md). |
| 5 | Documentation | Include documentation with public how-tos. | Docs with usage instructions. | Public how-tos in [`contracts/README.md`](../../contracts/README.md) (package layout, DKIM registry sourcing, env vars, deploy/verify command reference) and [`circom/README.md`](../../README.md) (`generate-example-contracts` for local Solidity), plus grant-facing per-deliverable evidence under [`milestone-2/`](./). **Proof:** [`05_documentation_and_howto.md`](./05_documentation_and_howto.md). |

## Current Status

- Project Setup: `Delivered`
- Local Environment: `Delivered`
- Verifier Interface and Wrappers: `Delivered`
- Template and Tooling: `Delivered`
- Documentation: `Delivered`

**Milestone 2:** `Delivered`

## Evidence Index

- [`01_project_setup.md`](./01_project_setup.md)
- [`02_local_environment.md`](./02_local_environment.md)
- [`03_verifier_interface_and_wrappers.md`](./03_verifier_interface_and_wrappers.md)
- [`04_template_and_tooling.md`](./04_template_and_tooling.md)
- [`05_documentation_and_howto.md`](./05_documentation_and_howto.md)

## Summary

- Hardhat + Foundry + resolc setup exists and is functional for target networks.
- Local EVM (`localEvm` / Anvil) and PolkaVM-compatible local RPC (`localPvm`) flows are documented with reproducible commands and captured outputs in [`02_local_environment.md`](./02_local_environment.md).
- Generic verifier-facing interfaces and a concrete Groth16 + DKIM wrapper are implemented; templates, Ignition deployment, and verification helpers cover EVM and PolkaVM-oriented targets.
- Additional proof systems beyond Groth16 are not implemented here; the interface layout is intended to allow future verifier backends without changing the wrapper’s DKIM integration pattern.
