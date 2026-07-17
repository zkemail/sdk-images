# 01 - Project Setup

Milestone 2 contracts project: a unified Hardhat + Foundry structure that compiles with both the native Solidity compiler (`solc`, EVM) and the PolkaVM Solidity compiler (`resolc`).

## Toolchain

- Hardhat is the primary build and deploy toolchain, using Ignition modules under `hh-ignition/`.
- Foundry is kept for test execution.
- The PolkaVM compilation path is provided via Hardhat + `@parity/hardhat-polkadot` + `resolc`.

## Build and Deployment Responsibilities

- Hardhat:
  - Compiles contracts (`solc` and `resolc` build targets).
  - Deploys with Ignition (`hh-ignition/modules/...`).
  - Runs verification where explorer APIs support it.
- Foundry:
  - Used for tests.
  - Not the primary deployment path for milestone delivery.

## Contracts Project Structure

- Hardhat config with PolkaVM/resolc support:
  - [`circom/contracts/hardhat.config.ts`](../../contracts/hardhat.config.ts)
- Foundry project config:
  - [`circom/contracts/foundry.toml`](../../contracts/foundry.toml)
- Build scripts/toolchain dependencies:
  - [`circom/contracts/package.json`](../../contracts/package.json)
- Ignition deployment module:
  - [`circom/contracts/hh-ignition/modules/ZKEmailVerifier.ts`](../../contracts/hh-ignition/modules/ZKEmailVerifier.ts)
- Contracts and interfaces:
  - [`circom/contracts/src/`](../../contracts/src/)

## Command Entry Points

- Compile: `yarn build`
- Deploy: `yarn deploy <chain_id>`
- Verify (where supported): `yarn verify chain-<chain_id>`

These scripts are defined in [`circom/contracts/package.json`](../../contracts/package.json).

## Related Documentation

- Contracts package overview and usage:
  - [`circom/contracts/README.md`](../../contracts/README.md)
- Grant index for milestone docs:
  - [`circom/kusama-grant/README.md`](../README.md)
