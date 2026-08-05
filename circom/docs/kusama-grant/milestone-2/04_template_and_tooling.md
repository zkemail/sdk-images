# 04 - Template and Tooling

Milestone 2 templating and tooling: reusable Tera templates for the wrapper and verifier contracts, plus Hardhat Ignition deploy and verify tooling for EVM and PolkaVM targets.

## Implementation Notes

### Solidity templates and generation

- **Tera sources** under [`circom/templates/`](../../templates/):
  - [`ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera): wrapper parameters (`signal_size`, `sender_domain`, regex / external-input metadata).
  - [`IGroth16Verifier.sol.tera`](../../templates/IGroth16Verifier.sol.tera): verifier interface sized to the blueprint’s public-signal count.
  - [`MockGroth16Verifier.sol.tera`](../../templates/MockGroth16Verifier.sol.tera): mock `Groth16Verifier.sol` for local / CI without a full `snarkjs` export.
- **Rust entrypoints** in [`circom/src/contract.rs`](../../src/contract.rs): render templates to disk, optionally run [`generate_verifier_contract`](../../src/contract.rs) (`snarkjs zkey export solidityverifier`) for a **real** Groth16 verifier when a `.zkey` is available.
- **Local / example path:** the `generate-example-contracts` CLI in [`circom/src/main.rs`](../../src/main.rs) (see [`02_local_environment.md`](./02_local_environment.md) and [`circom/README.md`](../../README.md)) writes mock verifier + wrapper + interface into [`circom/contracts/src/`](../../contracts/src/) without running the full pipeline.

### Deployment tooling

- **Hardhat Ignition:** [`circom/contracts/hh-ignition/modules/ZKEmailVerifier.ts`](../../contracts/hh-ignition/modules/ZKEmailVerifier.ts) deploys `Groth16Verifier` then `ZKEmailVerifier` with `DKIM_REGISTRY` from the environment. Invoked via `yarn deploy <network>` from [`circom/contracts/package.json`](../../contracts/package.json) (EVM and PolkaVM-facing networks are configured in [`hardhat.config.ts`](../../contracts/hardhat.config.ts)).

### Verification tooling

- **EVM:** `yarn verify chain-<chainId>` (Hardhat Ignition verify) for networks where an Etherscan-compatible API is configured and working (for example Base Sepolia).
- **PolkaVM:** source-code verification is not currently possible for PolkaVM deployments (e.g. Polkadot Hub testnet, `420420417`). The contract is `resolc`-compiled to PolkaVM/RISC-V bytecode; the Blockscout explorer's verification API and `@nomicfoundation/hardhat-verify` both only support EVM `solc`/Vyper bytecode, and `@parity/hardhat-polkadot` does not yet provide a `resolc`-aware verify task. This is a PolkaVM tooling gap, not a deployment issue: the contract remains fully visible on Blockscout (address, PolkaVM bytecode, transactions) and is exercisable via its read/write methods.
