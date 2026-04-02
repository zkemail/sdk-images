# 03 - Blueprint Deployment System

Deliverable mapping: Milestone 3, Deliverable 3 (`Blueprint Deployment System`).

## What must be delivered

- Configurable target chains for blueprint-driven runs.
- Automated contract generation and deployment for new blueprints (verifier address persisted for downstream consumers).

## Implementation Notes

- **Chain selection:** [`Payload.chain_id`](../../src/payload.rs) (numeric) is stringified for Hardhat Ignition (`yarn deploy <chain_id>`) inside [`deploy_verifier_contract`](../../src/contract.rs). It must match a **network key** in [`circom/contracts/hardhat.config.ts`](../../contracts/hardhat.config.ts) (e.g. `420420417`, `84532`, `11155111`). Named profiles such as `localPvm` / `localEvm` are for **manual** `yarn deploy` from [`circom/contracts`](../milestone-2/02_local_environment.md), not for the base64 pipeline payload field.
- **Secrets and registry:** The payload supplies `private_key`, `rpc_url`, `etherscan_api_key`, and `dkim_registry_address`, mapped into process env (`PRIVATE_KEY`, `RPC_URL`, …) before `yarn deploy`. [`hardhat.config.ts`](../../contracts/hardhat.config.ts) uses `RPC_URL` and `PRIVATE_KEY` for numeric networks’ `url` and `accounts`, so the pipeline’s RPC and signer are the ones Hardhat uses (defaults apply only when env is unset).
- **Automation steps:** For a standard pipeline run, `main` generates Solidity from blueprint data, exports Groth16 verifier from zkey, then deploys and reads `ZKEmailVerifierModule#ZKEmailVerifier` from Ignition’s `deployed_addresses.json` for that chain.
- **Persistence:** [`db::update_verifier_contract_address`](../../src/db.rs) writes the deployed address back to `blueprints.verifier_contract_address` for the blueprint UUID.
- **Per-blueprint artifacts:** There is no single global verifier address; each successful run produces chain-specific deployment records under `hh-ignition/deployments/chain-<id>/` inside the executed contracts workspace (`tmp/contracts` in CI/server flow).

## Repo Evidence

- Payload schema and env injection:
  - [`circom/src/payload.rs`](../../src/payload.rs)
- Deploy orchestration:
  - [`circom/src/contract.rs`](../../src/contract.rs) (`deploy_verifier_contract`, `read_ignition_deployed_address`)
- Pipeline call site:
  - [`circom/src/main.rs`](../../src/main.rs)
- Ignition module:
  - [`circom/contracts/hh-ignition/modules/ZKEmailVerifier.ts`](../../contracts/hh-ignition/modules/ZKEmailVerifier.ts)
- Network catalog:
  - [`circom/contracts/hardhat.config.ts`](../../contracts/hardhat.config.ts)
- DB update:
  - [`circom/src/db.rs`](../../src/db.rs)

## Evidence model

- Evidence is **run-based per blueprint** (and per target chain): contracts are generated and deployed when the pipeline runs for that blueprint.
- A single permanent verifier address is not required for the deliverable; reviewers look at reproducible pipeline behavior and stored DB + Ignition artifacts per run.

## Related documentation

- Milestone 2 [`02_local_environment.md`](../milestone-2/02_local_environment.md) — local chain profiles and manual deploy.
- Milestone 3 [`01_circom_pipeline_integration.md`](./01_circom_pipeline_integration.md) — how pipeline invokes deploy.

## Demonstration

From the **repository root**:

```bash
cargo test -p circom ignition_deployed_addresses
```

| Filter                        | Exercises                                                                                                                                                       |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ignition_deployed_addresses` | [`read_zkemail_verifier_from_deployed_addresses_path`](../../src/contract.rs) — Ignition `deployed_addresses.json` and `ZKEmailVerifierModule#ZKEmailVerifier`. |

The contracts bundle shape used before deploy is regression-tested in deliverable **01** ([`01_circom_pipeline_integration.md`](./01_circom_pipeline_integration.md)) via `test_contracts_bundle_and_zip_works`.

## Evidence standard for this deliverable

- Payload carries `chain_id` and registry/credentials; deploy and address readback are automated; verifier address is persisted for the blueprint record.

## Status

`Delivered`

Conclusion: Configurable `chain_id`, automated contract generation, Ignition deploy, and DB persistence of the verifier address are implemented in code.
