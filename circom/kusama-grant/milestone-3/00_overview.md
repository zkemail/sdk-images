# Milestone 3 - Verification Pipeline (Circom / contracts scope)

Primary Goal: Extend the blueprint pipeline to support automated contract generation, deployment, and on-chain verifier deployment for PolkaVM-oriented networks.

## Deliverables (this repository)

| # | Name | Description | Deliverables |
| --- | --- | --- | --- |
| 1 | Circom Pipeline Integration | Modify the existing Circom pipeline to integrate the new contracts project structure and support PolkaVM deployments. | Updated Circom pipeline capable of generating verifier contracts with Paseo testnet deployment. |
| 2 | Proof Formatting and Wrapper Integration Layer | Implement proof formatting, serialization, and integration logic to ensure compatibility between Circom-generated proofs and the verifier wrapper contracts, enabling seamless on-chain verification. | Proof formatting and integration layer enabling compatibility between Circom-generated proofs and on-chain verifier contracts. |
| 3 | Blueprint Deployment System | Extend the blueprint system to support configurable target chains and automate contract generation and deployment for new blueprints. | Blueprint pipeline supporting automatic contract generation and deployment. |

## Current Status

- Circom Pipeline Integration: `Delivered`
- Proof Formatting and Wrapper Integration Layer: `Delivered`
- Blueprint Deployment System: `Delivered`

**Milestone 3 (scope of this documentation):** `Delivered`

## Evidence index (this repository)

- [`01_circom_pipeline_integration.md`](./01_circom_pipeline_integration.md)
- [`02_proof_formatting_and_wrapper_integration.md`](./02_proof_formatting_and_wrapper_integration.md)
- [`03_blueprint_deployment_system.md`](./03_blueprint_deployment_system.md)

## Summary

The Circom pipeline generates templated wrappers and zkey-derived `Groth16Verifier`, deploys via Hardhat Ignition using `payload.chain_id` and env (including Polkadot Hub testnet `420420417`), and updates the blueprint verifier address in the application database. See the evidence index above.
