# 01 - Project Setup

Deliverable mapping: Milestone 2, Deliverable 1 (`Project Setup`).

## What must be delivered

- Unified project structure using Hardhat and Foundry.
- Support for both `solc` and `resolc` compilation.

## Implementation Notes

- Hardhat is the primary build/deploy toolchain for this package.
- Foundry is kept for test execution.
- Deployment uses Hardhat Ignition modules.
- PolkaVM compilation path is provided via Hardhat + `@parity/hardhat-polkadot` + `resolc`.

## Build and Deployment Responsibilities

- Hardhat:
  - Compiles contracts.
  - Handles `solc` and `resolc`-based build targets.
  - Deploys with Ignition (`hh-ignition/modules/...`).
  - Runs verification where explorer APIs support it.
- Foundry:
  - Used for tests.
  - Not the primary deployment path for milestone delivery.

## Verification Limitation (Paseo)

- RouteScan can be used for manual verification on Paseo/Polkadot Hub explorer environments.
- The Hardhat verification config example currently provided by RouteScan does not work in this setup (API error is returned).
- This indicates the explorer endpoint is not yet behaving as a standard Etherscan-compatible API path for automated verification in our workflow.
- As a result, automated verification is treated as unavailable for Paseo at the moment; manual RouteScan verification is the practical path.

## Contracts Project Structure

- Hardhat config with PolkaVM/resolc support:
  - `circom/contracts/hardhat.config.ts`
- Foundry project config:
  - `circom/contracts/foundry.toml`
- Build scripts/toolchain dependencies:
  - `circom/contracts/package.json`
- Ignition deployment module:
  - `circom/contracts/hh-ignition/modules/ZKEmailVerifier.ts`
- Contracts and interfaces:
  - `circom/contracts/src/`

## Command Entry Points

- Compile: `yarn build`
- Deploy: `yarn deploy <chain_id>`
- Verify (where supported): `yarn verify chain-<chain_id>`

These scripts are defined in `circom/contracts/package.json`.

## Related Documentation

- Contracts package overview and usage:
  - `circom/contracts/README.md`
- Grant index for milestone docs:
  - `circom/kusama-grant/README.md`

## Status

`Delivered`

Conclusion: The deliverable "Project structure with Hardhat and Foundry configuration supporting EVM and PolkaVM compilation." is delivered.
