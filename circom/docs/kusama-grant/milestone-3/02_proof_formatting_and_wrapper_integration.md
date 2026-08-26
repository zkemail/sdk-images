# 02 - Proof Formatting and Wrapper Integration Layer

Milestone 3 proof-formatting integration: the templated `ZKEmailVerifier` wrapper decodes and verifies Groth16 proofs produced by the pipeline for a given blueprint.

## Implementation Notes

- **On-chain formatting:** The generated `ZKEmailVerifier` (from [`circom/templates/ZKEmailVerifier.sol.tera`](../../../templates/ZKEmailVerifier.sol.tera)) implements [`IZKEmailVerifier`](../../../contracts/src/interfaces/IZKEmailVerifier.sol):
  - Validates `publicInputs.length` against the blueprint-derived `PUBLIC_INPUTS_LENGTH`.
  - Reads the DKIM public key hash at `PUBLIC_KEY_HASH_OFFSET` and checks `IDKIMRegistry.isKeyHashValid(DOMAIN_HASH, keyHash)`.
  - **Decodes** calldata `proof` as Groth16 `(uint256[2] pA, uint256[2][2] pB, uint256[2] pC)` via `abi.decode`, matching the encoding provers typically produce for Solidity verifiers.
  - **Converts** `bytes32[]` public inputs to `uint256[PUBLIC_INPUTS_LENGTH]` for `IGroth16Verifier.verifyProof`.
- **Pipeline alignment:** The same `ContractData` / `signal_size` logic in [`circom/src/contract.rs`](../../../src/contract.rs) (`prepare_contract_data`, `create_zkemail_verifier_*`, `generate_verifier_contract`) drives template rendering so public-signal layout matches the circuit; `Groth16Verifier.sol` is exported from the blueprint zkey so `verifyProof` matches client-generated proofs. Full pipeline ordering (proof system + deploy) is in [`circom/src/main.rs`](../../../src/main.rs).
- **Committed integrator-facing API:** [`circom/contracts/src/interfaces/IZKEmailVerifier.sol`](../../../contracts/src/interfaces/IZKEmailVerifier.sol) (generated wrapper output itself is not committed; see root `.gitignore`).
- **Milestone 2 reference:** Interface semantics and committed-vs-generated layout are documented in Milestone 2 [`03_verifier_interface_and_wrappers.md`](../milestone-2/03_verifier_interface_and_wrappers.md); Tera templates and `snarkjs` verifier export are documented in [`04_template_and_tooling.md`](../milestone-2/04_template_and_tooling.md).

## Demonstration

From the **repository root** (Cargo workspace). No full circuit compile is required.

```bash
cargo test -p circom test_contracts_bundle_and_zip_works
```

| Filter | Exercises |
| --- | --- |
| `test_contracts_bundle_and_zip_works` | [`create_zkemail_verifier_and_interface_at_paths`](../../../src/contract.rs) / [`create_mock_groth16_verifier_at_path`](../../../src/contract.rs): renders the wrapper and verifier Solidity from `ContractData` (signal size, public-key-hash offset) -- the formatting/compatibility step this deliverable covers. `prepare_contract_data`, which computes those fields from a real payload and circuit, is unit-tested and cited separately in Milestone 2 [`03_verifier_interface_and_wrappers.md`](../milestone-2/03_verifier_interface_and_wrappers.md). |

Same test as Milestone 3 [`01_circom_pipeline_integration.md`](./01_circom_pipeline_integration.md), which now runs in CI via [`circom-rust-tests.yml`](../../../../.github/workflows/circom-rust-tests.yml). Example passing `test` job: [`manifest.json#L7-L11`](./manifest.json#L7-L11). For the current state of the branch, see the [Actions tab](https://github.com/zkemail/sdk-images/actions/workflows/circom-rust-tests.yml?query=branch%3Astaging).
