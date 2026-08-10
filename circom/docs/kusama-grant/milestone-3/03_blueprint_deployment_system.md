# 03 - Blueprint Deployment System

Milestone 3 blueprint deployment: the blueprint pipeline selects a target chain from the payload and automates contract generation, deployment, and address persistence for that blueprint.

## Implementation Notes

- **Chain selection:** [`Payload.chain_id`](../../../src/payload.rs) (numeric) is stringified for Hardhat Ignition (`yarn deploy <chain_id>`) inside [`deploy_verifier_contract`](../../../src/contract.rs). It must match a **network key** in [`circom/contracts/hardhat.config.ts`](../../../contracts/hardhat.config.ts) (e.g. `420420417`, `84532`, `11155111`). Named profiles such as `localPvm` / `localEvm` are for **manual** `yarn deploy` from [`circom/contracts`](../milestone-2/02_local_environment.md), not for the base64 pipeline payload field.
- **Secrets and registry:** The payload supplies `private_key`, `rpc_url`, `etherscan_api_key`, and `dkim_registry_address`, mapped into process env (`PRIVATE_KEY`, `RPC_URL`, …) before `yarn deploy`. [`hardhat.config.ts`](../../../contracts/hardhat.config.ts) uses `RPC_URL` and `PRIVATE_KEY` for numeric networks’ `url` and `accounts`, so the pipeline’s RPC and signer are the ones Hardhat uses (defaults apply only when env is unset).
- **Automation steps:** For a standard pipeline run (see Milestone 3 [`01_circom_pipeline_integration.md`](./01_circom_pipeline_integration.md)), `main` generates Solidity from blueprint data, exports Groth16 verifier from zkey, then deploys via the [Ignition module](../../../contracts/hh-ignition/modules/ZKEmailVerifier.ts) and reads `ZKEmailVerifierModule#ZKEmailVerifier` from Ignition's `deployed_addresses.json` for that chain.
- **Persistence:** [`db::update_verifier_contract_address`](../../../src/db.rs) writes the deployed address back to `blueprints.verifier_contract_address` for the blueprint UUID.
- **Per-blueprint artifacts:** There is no single global verifier address; each successful run produces chain-specific deployment records under `hh-ignition/deployments/chain-<id>/` inside the executed contracts workspace (`tmp/contracts` in CI/server flow). Evidence is run-based per blueprint and target chain, not a single permanent verifier address.

## Demonstration

From the **repository root**:

```bash
cargo test -p circom ignition_deployed_addresses
```

| Filter                        | Exercises                                                                                                                                                       |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ignition_deployed_addresses` | [`read_zkemail_verifier_from_deployed_addresses_path`](../../../src/contract.rs): Ignition `deployed_addresses.json` and `ZKEmailVerifierModule#ZKEmailVerifier`. |

The contracts bundle shape used before deploy is regression-tested in deliverable **01** ([`01_circom_pipeline_integration.md`](./01_circom_pipeline_integration.md)) via `test_contracts_bundle_and_zip_works`.

Runs in CI on every push via [`.github/workflows/circom-rust-tests.yml`](../../../../.github/workflows/circom-rust-tests.yml) (`contract::` scope). Example passing `test` job (2026-07-27): https://github.com/zkemail/sdk-images/actions/runs/30265748673/job/89975888081. For the current state of the branch, see the [Actions tab](https://github.com/zkemail/sdk-images/actions/workflows/circom-rust-tests.yml?query=branch%3Astaging).

A real pipeline-triggered deployment: the [`kusama_grant_paseo_e2e` blueprint](https://registry-staging.onrender.com/e94e7f93-7575-4e26-a147-de894b19ce3e/versions) (`e94e7f93-7575-4e26-a147-de894b19ce3e`) was created with Target Chain set to Paseo Testnet, compiled and deployed by this pipeline, and its record carries the persisted result (`verifier_contract_chain` `420420417`, `verifier_contract_address` [`0x72616B78d29d0cccBfEec1bf00E108885286D2f3`](https://blockscout-testnet.polkadot.io/address/0x72616B78d29d0cccBfEec1bf00E108885286D2f3)). The deployed bytecode is reproduced from that blueprint's own `circuit.zip` in [`07_bytecode_verification.md`](https://github.com/zkemail/registry/blob/kusama-grant/docs/kusama-grant/milestone-3/07_bytecode_verification.md).
