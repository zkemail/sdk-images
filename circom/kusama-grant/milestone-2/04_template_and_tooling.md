# 04 - Template and Tooling

Deliverable mapping: Milestone 2, Deliverable 4 (`Template and Tooling`).

## What must be delivered

- Wrapper contracts converted into reusable templates.
- Deployment and verification tooling for both environments.

## Implementation Notes

### Solidity templates and generation

- **Tera sources** under [`circom/templates/`](../../templates/):
  - [`ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera): wrapper parameters (`signal_size`, `sender_domain`, regex / external-input metadata).
  - [`IGroth16Verifier.sol.tera`](../../templates/IGroth16Verifier.sol.tera): verifier interface sized to the blueprint’s public-signal count.
  - [`MockGroth16Verifier.sol.tera`](../../templates/MockGroth16Verifier.sol.tera): mock `Groth16Verifier.sol` for local / CI without a full `snarkjs` export.
- **Rust entrypoints** in [`circom/src/contract.rs`](../../src/contract.rs): render templates to disk, optionally run [`generate_verifier_contract`](../../src/contract.rs) (`snarkjs zkey export solidityverifier`) for a **real** Groth16 verifier when a `.zkey` is available.
- **Local / example path:** `cargo run -p circom -- generate-example-contracts …` (see [`02_local_environment.md`](./02_local_environment.md) and [`circom/README.md`](../../README.md)) writes mock verifier + wrapper + interface into `circom/contracts/src/` without running the full pipeline.

### Circuit template

- [`circom/templates/template.circom.tera`](../../templates/template.circom.tera) is rendered by [`circom/src/template.rs`](../../src/template.rs) for blueprint-specific Circom sources (broader pipeline; complements Solidity templating).

### Downloadable contract bundle

- The Circom binary maintains a fixed list of paths (`CONTRACT_BUNDLE_FILES` in [`circom/src/main.rs`](../../src/main.rs)) copied or generated when producing the user-facing contracts zip: Hardhat/Ignition layout, static interfaces, and **generated** `src/*.sol` files produced from templates or `snarkjs` as appropriate.

### Deployment tooling

- **Hardhat Ignition:** [`circom/contracts/hh-ignition/modules/ZKEmailVerifier.ts`](../../contracts/hh-ignition/modules/ZKEmailVerifier.ts) deploys `Groth16Verifier` then `ZKEmailVerifier` with `DKIM_REGISTRY` from the environment. Invoked via `yarn deploy <network>` from [`circom/contracts/package.json`](../../contracts/package.json) (EVM and PolkaVM-facing networks are configured in [`hardhat.config.ts`](../../contracts/hardhat.config.ts)).

### Verification tooling

- **EVM:** `yarn verify chain-<chainId>` (Hardhat Ignition verify) for networks where an Etherscan-compatible API is configured and working (for example Base Sepolia).
- **PolkaVM:** source-code verification is not currently possible for PolkaVM deployments (e.g. Polkadot Hub testnet, `420420417`). The contract is `resolc`-compiled to PolkaVM/RISC-V bytecode; the Blockscout explorer's verification API and `@nomicfoundation/hardhat-verify` both only support EVM `solc`/Vyper bytecode, and `@parity/hardhat-polkadot` does not yet provide a `resolc`-aware verify task. This is a PolkaVM tooling gap, not a deployment issue: the contract remains fully visible on Blockscout (address, PolkaVM bytecode, transactions) and is exercisable via its read/write methods.

## Repo Evidence

- Reusable Solidity templates:
  - [`circom/templates/ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera)
  - [`circom/templates/IGroth16Verifier.sol.tera`](../../templates/IGroth16Verifier.sol.tera)
  - [`circom/templates/MockGroth16Verifier.sol.tera`](../../templates/MockGroth16Verifier.sol.tera)
- Circuit template (same Tera stack):
  - [`circom/templates/template.circom.tera`](../../templates/template.circom.tera)
- Generation and optional real verifier export:
  - [`circom/src/contract.rs`](../../src/contract.rs)
  - [`circom/src/main.rs`](../../src/main.rs) (`generate-example-contracts`, bundle assembly)
- Deployment and verification:
  - Hardhat Ignition module [`circom/contracts/hh-ignition/modules/ZKEmailVerifier.ts`](../../contracts/hh-ignition/modules/ZKEmailVerifier.ts), driven by `yarn deploy` / `yarn verify` in [`circom/contracts/package.json`](../../contracts/package.json)
- Operator-facing summaries:
  - [`circom/contracts/README.md`](../../contracts/README.md)

## Related documentation

- Wrapper / interface intent and what is generated vs committed: [`03_verifier_interface_and_wrappers.md`](./03_verifier_interface_and_wrappers.md)
- Local prerequisites (`generate-example-contracts`, `yarn build`, deploy): [`02_local_environment.md`](./02_local_environment.md)

## Evidence standard for this deliverable

- Templates exist for the wrapper, Groth16 interface, and mock verifier; the pipeline can emit a real Groth16 verifier from a zkey when provided.
- Deployment is reproducible via Ignition; verification is covered by Ignition verify, with the PolkaVM verification limitation documented in the Verification tooling section above.

## Status

`Delivered`

Conclusion: Reusable Tera templates and Rust generation cover Solidity wrappers and related verifier artifacts; Hardhat Ignition plus Foundry script and verify helpers provide deployment and verification tooling across EVM-oriented and PolkaVM-configured Hardhat networks, with documented caveats where automated verification is not yet reliable.
