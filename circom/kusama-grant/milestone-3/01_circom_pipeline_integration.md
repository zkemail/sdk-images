# 01 - Circom Pipeline Integration

Deliverable mapping: Milestone 3, Deliverable 1 (`Circom Pipeline Integration`).

## What must be delivered

- Integrate the Circom pipeline with the contracts project structure (Hardhat/Ignition layout, generated Solidity).
- Support PolkaVM-oriented deployments from the pipeline (via Hardhat Polkadot networks, e.g. Polkadot Hub testnet chain id `420420417`).

## Implementation Notes

- **End-to-end orchestration** in [`circom/src/main.rs`](../../src/main.rs): after circuit build and key material are produced, the pipeline (1) renders `ZKEmailVerifier.sol` and `IGroth16Verifier.sol` from Tera templates, (2) exports `Groth16Verifier.sol` from the zkey via `snarkjs`, (3) runs cleanup/upload of artifacts, (4) calls [`deploy_verifier_contract`](../../src/contract.rs) with `payload.chain_id`, and (5) persists the deployed verifier address via [`update_verifier_contract_address`](../../src/db.rs) against the running database.
- **Environment wiring for deploy:** [`circom/src/payload.rs`](../../src/payload.rs) maps payload fields into process env (`PRIVATE_KEY`, `RPC_URL`, `CHAIN_ID`, `ETHERSCAN_API_KEY`, `DKIM_REGISTRY`) before Hardhat runs inside `tmp/contracts`.
- **Contracts layout:** `CONTRACT_BUNDLE_FILES` in [`main.rs`](../../src/main.rs) defines the zip/deploy tree (Hardhat config, Ignition module, static interfaces, generated `src/*.sol`).
- **Network definitions:** [`circom/contracts/hardhat.config.ts`](../../contracts/hardhat.config.ts) includes PolkaVM targets (`420420417`, embedded `hardhat` / `localPvm` profiles), EVM testnets (`84532`, `11155111`), and named local profiles (`localEvm`, `localPvm`). For numeric keys, `RPC_URL` and `PRIVATE_KEY` from the environment override the default public RPC and supply `accounts` (aligned with [zk-email-verify](https://github.com/zkemail/zk-email-verify/blob/kusama-grant/packages/contracts/hardhat.config.ts)), so payload-injected env matches what Hardhat reads. Ignition uses `--network <chain_id>` as passed from `yarn deploy <chain_id>`.
- **Verification behavior:** For `chain_id == 420420417`, [`deploy_verifier_contract`](../../src/contract.rs) skips automated explorer verification (same PolkaVM verification constraint documented in [`milestone-2/04_template_and_tooling.md`](../milestone-2/04_template_and_tooling.md)).

## Repo Evidence

- Pipeline orchestration and bundle list:
  - [`circom/src/main.rs`](../../src/main.rs)
- Deploy invocation, Ignition address readout, optional `snarkjs` verifier export:
  - [`circom/src/contract.rs`](../../src/contract.rs)
- Payload → env mapping:
  - [`circom/src/payload.rs`](../../src/payload.rs)
- Post-deploy DB update:
  - [`circom/src/db.rs`](../../src/db.rs)
- Hardhat / PolkaVM network configuration:
  - [`circom/contracts/hardhat.config.ts`](../../contracts/hardhat.config.ts)

## Related documentation

- Template and generation details: Milestone 2 [`04_template_and_tooling.md`](../milestone-2/04_template_and_tooling.md)
- Local reproduction (generate contracts, `localPvm`): Milestone 2 [`02_local_environment.md`](../milestone-2/02_local_environment.md)

## Demonstration

From the **repository root** (Cargo workspace). No full circuit compile is required.

```bash
cargo test -p circom hardhat_env_from_payload
cargo test -p circom test_contracts_bundle_and_zip_works
```

| Filter | Exercises |
| --- | --- |
| `hardhat_env_from_payload` | [`hardhat_deploy_env_from_payload`](../../src/payload.rs): deploy env (`PRIVATE_KEY`, `RPC_URL`, `CHAIN_ID`, etc.). |
| `test_contracts_bundle_and_zip_works` | Template-generated Solidity and [`CONTRACT_BUNDLE_FILES`](../../src/main.rs) zip layout (contracts tree the pipeline emits). |

## Evidence standard for this deliverable

- Code paths show contracts generation integrated into the same run as circuit artifacts and deployment targeting a network id from the payload.
- Hardhat config defines the Paseo / Polkadot Hub testnet profile (`420420417`) so the pipeline **can** deploy there when that `chain_id` and credentials are supplied at runtime.

## Status

`Delivered`

Conclusion: The Circom pipeline integrates the contracts project structure and deploys through Ignition using `payload.chain_id` (including Polkadot Hub testnet `420420417`), persisting the verifier address via the application database.
