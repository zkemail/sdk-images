# Milestone 2 - ZK Verifier Contract Tooling

Primary Goal: Develop a reusable smart contract project with tooling supporting both EVM and PolkaVM deployments, including wrapper contract templates that integrate DKIM registry verification.

## Deliverables

| # | Name | Description | Deliverable | What was done / Proof |
| --- | --- | --- | --- | --- |
| 1 | Project Setup | Create a unified project structure using Hardhat and Foundry. Configure support for both the native Solidity compiler (`solc`) and the PolkaVM Solidity compiler (`resolc`). | Project structure with Hardhat and Foundry configuration supporting EVM and PolkaVM compilation. | Unified Hardhat + Foundry package with `solc` and `resolc` build targets. **Proof:** [`01_project_setup.md`](./01_project_setup.md). |
| 2 | Local Environment | Configure local testing environments for both EVM and PolkaVM, enabling contract deployment and testing on local nodes. | Working local setup capable of deploying contracts to Anvil and a local PolkaVM compatible node. | `localEvm` (Anvil) and `localPvm` networks, with reproducible build and Ignition-deploy commands and captured outputs for both. **Proof:** [`02_local_environment.md`](./02_local_environment.md). |
| 3 | Verifier Interface and Wrappers | Define a generic verifier interface and implement wrapper contracts for Groth16 that integrate DKIM verification and prepare/format proofs before invoking the underlying verifier contract. | Verifier interface and wrapper contracts compatible with different proof systems. | Generic `IZKEmailVerifier` interface plus a per-blueprint Groth16 + DKIM wrapper that validates the DKIM key hash and formats the proof before verifying. A committed test (`test/TestBlueprintZKEmailVerifier.t.sol`) runs a real Groth16 proof through a concrete wrapper instance and a real (non-mock) verifier end-to-end, running in CI on every push. **Proof:** [`03_verifier_interface_and_wrappers.md`](./03_verifier_interface_and_wrappers.md). |
| 4 | Template and Tooling | Convert wrapper contracts into reusable templates and provide deployment and verification tooling for both environments. | Templated wrapper contracts, deployment scripts, and verification tooling. | Reusable Tera templates for the wrapper and verifier, with Hardhat Ignition deploy (`yarn deploy`) and `yarn verify`. **Proof:** [`04_template_and_tooling.md`](./04_template_and_tooling.md). |
| 5 | Documentation | Include documentation with public how-tos. | Docs with usage instructions. | Public how-to with a runnable local generate/build/deploy flow, plus the contracts and Circom READMEs. **Proof:** [`05_public_howto.md`](./05_public_howto.md). |

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
- [`05_public_howto.md`](./05_public_howto.md)
- [`06_e2e_demo.md`](./06_e2e_demo.md) -- additional evidence added in response to review, not itself a milestone deliverable

## Summary

- Hardhat + Foundry + resolc setup exists and is functional for target networks.
- Local EVM (`localEvm` / Anvil) and PolkaVM-compatible local RPC (`localPvm`) flows are documented with reproducible commands and captured outputs in [`02_local_environment.md`](./02_local_environment.md).
- Generic verifier-facing interfaces and a concrete Groth16 + DKIM wrapper are implemented; templates, Ignition deployment, and verification helpers cover EVM and PolkaVM-oriented targets.
- Additional proof systems beyond Groth16 are not implemented here; the interface layout is intended to allow future verifier backends without changing the wrapper’s DKIM integration pattern.
- Contract tests (including the real-proof fixture test above) run in CI on every push via [`.github/workflows/circom-contracts-tests.yml`](../../../.github/workflows/circom-contracts-tests.yml). Example passing `run_contracts_tests` job (2026-07-27): https://github.com/zkemail/sdk-images/actions/runs/30265748763/job/89975888650. For the current state of the branch, see the [Actions tab](https://github.com/zkemail/sdk-images/actions/workflows/circom-contracts-tests.yml?query=branch%3Astaging).
- The Rust tooling that generates the wrapper/verifier (`prepare_contract_data` in `circom/src/contract.rs`) is unit-tested in CI via [`.github/workflows/circom-rust-tests.yml`](../../../.github/workflows/circom-rust-tests.yml). Example passing `test` job (2026-07-27): https://github.com/zkemail/sdk-images/actions/runs/30265748673/job/89975888081. For the current state of the branch, see the [Actions tab](https://github.com/zkemail/sdk-images/actions/workflows/circom-rust-tests.yml?query=branch%3Astaging).
- As a community-facing outcome, the generic project structure and dual-target (EVM + PolkaVM) tooling are published as a standalone GitHub template repository, [`zkemail/polkavm-hardhat-template`](https://github.com/zkemail/polkavm-hardhat-template) (verifier/DKIM-specific contracts stripped, minimal `Counter` example), so anyone can click "Use this template" and deploy Solidity to both EVM and PolkaVM. See [`05_public_howto.md`](./05_public_howto.md) for details.
