# 01 - Circom Pipeline Integration

Milestone 3 pipeline integration: the Circom pipeline generates verifier contracts from the contracts project structure and deploys them via Hardhat Ignition, including to PolkaVM-oriented networks.

## Implementation Notes

- **End-to-end orchestration** in [`circom/src/main.rs`](../../../src/main.rs): after circuit build and key material are produced, the pipeline (1) renders `ZKEmailVerifier.sol` and `IGroth16Verifier.sol` from Tera templates, (2) exports `Groth16Verifier.sol` from the zkey via `snarkjs`, (3) runs cleanup/upload of artifacts, (4) calls [`deploy_verifier_contract`](../../../src/contract.rs) with `payload.chain_id`, and (5) persists the deployed verifier address via [`update_verifier_contract_address`](../../../src/db.rs) against the running database.
- **Environment wiring for deploy:** [`circom/src/payload.rs`](../../../src/payload.rs) maps payload fields into process env (`PRIVATE_KEY`, `RPC_URL`, `CHAIN_ID`, `ETHERSCAN_API_KEY`, `DKIM_REGISTRY`) before Hardhat runs inside `tmp/contracts`.
- **Contracts layout:** `CONTRACT_BUNDLE_FILES` in [`main.rs`](../../../src/main.rs) defines the zip/deploy tree (Hardhat config, Ignition module, static interfaces, generated `src/*.sol`).
- **Network definitions:** [`circom/contracts/hardhat.config.ts`](../../../contracts/hardhat.config.ts) includes PolkaVM targets (`420420417`, embedded `hardhat` / `localPvm` profiles), EVM testnets (`84532`, `11155111`), and named local profiles (`localEvm`, `localPvm`). For numeric keys, `RPC_URL` and `PRIVATE_KEY` from the environment override the default public RPC and supply `accounts` (aligned with [zk-email-verify](https://github.com/zkemail/zk-email-verify/blob/kusama-grant/packages/contracts/hardhat.config.ts)), so payload-injected env matches what Hardhat reads. Ignition uses `--network <chain_id>` as passed from `yarn deploy <chain_id>`.
- **Verification behavior:** For `chain_id == 420420417`, [`deploy_verifier_contract`](../../../src/contract.rs) skips automated explorer verification (same PolkaVM verification constraint documented in Milestone 2 [`04_template_and_tooling.md`](../milestone-2/04_template_and_tooling.md)).
- **Local reproduction:** see Milestone 2 [`02_local_environment.md`](../milestone-2/02_local_environment.md) for generating contracts and deploying to `localPvm`.

## Demonstration

From the **repository root** (Cargo workspace). No full circuit compile is required.

```bash
cargo test -p circom hardhat_env_from_payload
cargo test -p circom test_contracts_bundle_and_zip_works
```

| Filter | Exercises |
| --- | --- |
| `hardhat_env_from_payload` | [`hardhat_deploy_env_from_payload`](../../../src/payload.rs): deploy env (`PRIVATE_KEY`, `RPC_URL`, `CHAIN_ID`, etc.). |
| `test_contracts_bundle_and_zip_works` | Template-generated Solidity and [`CONTRACT_BUNDLE_FILES`](../../../src/main.rs) zip layout (contracts tree the pipeline emits). |

Runs in CI on every push via [`.github/workflows/circom-rust-tests.yml`](../../../../.github/workflows/circom-rust-tests.yml) (`payload::` and this named test, added alongside `contract::`). Example passing `test` job (2026-08-03): https://github.com/zkemail/sdk-images/actions/runs/30817776514/job/91699698934. For the current state of the branch, see the [Actions tab](https://github.com/zkemail/sdk-images/actions/workflows/circom-rust-tests.yml?query=branch%3Astaging).
