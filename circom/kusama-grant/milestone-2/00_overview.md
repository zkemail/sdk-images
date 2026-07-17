# Milestone 2 - ZK Verifier Contract Tooling

Primary Goal: Develop a reusable smart contract project with tooling supporting both EVM and PolkaVM deployments, including wrapper contract templates that integrate DKIM registry verification.

## Deliverables

| # | Name | Description | Deliverable | What was done / Proof |
| --- | --- | --- | --- | --- |
| 1 | Project Setup | Create a unified project structure using Hardhat and Foundry. Configure support for both the native Solidity compiler (`solc`) and the PolkaVM Solidity compiler (`resolc`). | Project structure with Hardhat and Foundry configuration supporting EVM and PolkaVM compilation. | Unified Hardhat + Foundry package with `solc` and `resolc` build targets. **Proof:** [`01_project_setup.md`](./01_project_setup.md). |
| 2 | Local Environment | Configure local testing environments for both EVM and PolkaVM, enabling contract deployment and testing on local nodes. | Working local setup capable of deploying contracts to Anvil and a local PolkaVM compatible node. | `localEvm` (Anvil) and `localPvm` networks, with reproducible build and Ignition-deploy commands and captured outputs for both. **Proof:** [`02_local_environment.md`](./02_local_environment.md). |
| 3 | Verifier Interface and Wrappers | Define a generic verifier interface and implement wrapper contracts for Groth16 that integrate DKIM verification and prepare/format proofs before invoking the underlying verifier contract. | Verifier interface and wrapper contracts compatible with different proof systems. | Generic `IZKEmailVerifier` interface plus a per-blueprint Groth16 + DKIM wrapper that validates the DKIM key hash and formats the proof before verifying. **Proof:** [`03_verifier_interface_and_wrappers.md`](./03_verifier_interface_and_wrappers.md). |
| 4 | Template and Tooling | Convert wrapper contracts into reusable templates and provide deployment and verification tooling for both environments. | Templated wrapper contracts, deployment scripts, and verification tooling. | Reusable Tera templates for the wrapper and verifier, with Hardhat Ignition deploy (`yarn deploy`) and `yarn verify`. **Proof:** [`04_template_and_tooling.md`](./04_template_and_tooling.md). |
| 5 | Documentation | Include documentation with public how-tos. | Docs with usage instructions. | Public how-tos in the contracts and Circom READMEs, plus per-deliverable grant evidence. **Proof:** [`05_documentation_and_howto.md`](./05_documentation_and_howto.md). |

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
