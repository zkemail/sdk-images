# Milestone 3

Primary Goal: extend the Circom pipeline, SDK, and registry frontend to support automated PolkaVM verifier contract generation and deployment, and end-to-end on-chain proof verification.

Milestone 3 spans five deliverables across three repositories:

| # | Name | Repository | Evidence |
| --- | --- | --- | --- |
| 1 | Circom Pipeline Integration | sdk-images | [`01_circom_pipeline_integration.md`](https://github.com/zkemail/sdk-images/blob/kusama-grant/circom/docs/kusama-grant/milestone-3/01_circom_pipeline_integration.md) |
| 2 | Proof Formatting and Wrapper Integration Layer | sdk-images | [`02_proof_formatting_and_wrapper_integration.md`](https://github.com/zkemail/sdk-images/blob/kusama-grant/circom/docs/kusama-grant/milestone-3/02_proof_formatting_and_wrapper_integration.md) |
| 3 | Blueprint Deployment System | sdk-images | [`03_blueprint_deployment_system.md`](https://github.com/zkemail/sdk-images/blob/kusama-grant/circom/docs/kusama-grant/milestone-3/03_blueprint_deployment_system.md) |
| 4 | SDK On-Chain Verification | zk-email-sdk-js | [`04_sdk_on_chain_verification.md`](https://github.com/zkemail/zk-email-sdk-js/blob/kusama-grant/docs/kusama-grant/milestone-3/04_sdk_on_chain_verification.md) |
| 5 | Frontend Integration & Documentation | registry | [`05_frontend_integration_and_documentation.md`](https://github.com/zkemail/registry/blob/kusama-grant/docs/kusama-grant/milestone-3/05_frontend_integration_and_documentation.md) |

## In this repository (sdk-images)

Deliverables 1-3 (Circom/contracts scope):

| # | Name | Description | Deliverable | What was done / Proof |
| --- | --- | --- | --- | --- |
| 1 | Circom Pipeline Integration | Modify the existing Circom pipeline to integrate the new contracts project structure and support PolkaVM deployments. | Updated Circom pipeline capable of generating verifier contracts with Paseo testnet deployment. | Pipeline orchestration in `main.rs` renders templates, exports the verifier, deploys via Ignition using `payload.chain_id` (including Polkadot Hub testnet `420420417`), and persists the address to the database. **Proof:** [`01_circom_pipeline_integration.md`](./01_circom_pipeline_integration.md). |
| 2 | Proof Formatting and Wrapper Integration Layer | Implement proof formatting, serialization, and integration logic to ensure compatibility between Circom-generated proofs and the verifier wrapper contracts, enabling seamless on-chain verification. | Proof formatting and integration layer enabling compatibility between Circom-generated proofs and on-chain verifier contracts. | Templated `ZKEmailVerifier` wrapper decodes Groth16 proofs and public inputs; the wrapper/verifier Solidity itself is rendered from `ContractData` (signal size, public-key-hash offset) via `create_zkemail_verifier_and_interface_at_paths`, gated on the DKIM key hash. **Proof:** [`02_proof_formatting_and_wrapper_integration.md`](./02_proof_formatting_and_wrapper_integration.md). |
| 3 | Blueprint Deployment System | Extend the blueprint system to support configurable target chains and automate contract generation and deployment for new blueprints. | Blueprint pipeline supporting automatic contract generation and deployment. | Chain id from the blueprint payload selects the Ignition network; the deployed verifier address is read back from Ignition's output and persisted to the blueprint record. **Proof:** [`03_blueprint_deployment_system.md`](./03_blueprint_deployment_system.md). |

Local evidence index:

- [`01_circom_pipeline_integration.md`](./01_circom_pipeline_integration.md)
- [`02_proof_formatting_and_wrapper_integration.md`](./02_proof_formatting_and_wrapper_integration.md)
- [`03_blueprint_deployment_system.md`](./03_blueprint_deployment_system.md)

## Summary

The Circom pipeline generates templated wrappers and zkey-derived `Groth16Verifier`, deploys via Hardhat Ignition using `payload.chain_id` and env (including Polkadot Hub testnet `420420417`), and updates the blueprint verifier address in the application database. See the evidence index above.
