# 02 - Proof Formatting and Wrapper Integration Layer

Deliverable mapping: Milestone 3, Deliverable 2 (`Proof Formatting and Wrapper Integration Layer`).

## What must be delivered

- Proof formatting / serialization compatibility between Circom-generated Groth16 proofs and the on-chain wrapper.
- A clear on-chain verification path via the wrapper (`IZKEmailVerifier.verify`) and underlying `Groth16Verifier`.

## Implementation Notes

- **On-chain formatting:** The generated `ZKEmailVerifier` (from [`circom/templates/ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera)) implements [`IZKEmailVerifier`](../../contracts/src/interfaces/IZKEmailVerifier.sol):
  - Validates `publicInputs.length` against the blueprint-derived `PUBLIC_INPUTS_LENGTH`.
  - Reads the DKIM public key hash at `PUBLIC_KEY_HASH_OFFSET` and checks `IDKIMRegistry.isKeyHashValid(DOMAIN_HASH, keyHash)`.
  - **Decodes** calldata `proof` as Groth16 `(uint256[2] pA, uint256[2][2] pB, uint256[2] pC)` via `abi.decode`, matching the encoding provers typically produce for Solidity verifiers.
  - **Converts** `bytes32[]` public inputs to `uint256[PUBLIC_INPUTS_LENGTH]` for `IGroth16Verifier.verifyProof`.
- **Pipeline alignment:** The same `ContractData` / `signal_size` logic in [`circom/src/contract.rs`](../../src/contract.rs) (`prepare_contract_data`) drives template rendering so public-signal layout matches the circuit; `Groth16Verifier.sol` is exported from the blueprint zkey so `verifyProof` matches client-generated proofs.
- **Milestone 2 reference:** Interface semantics and committed-vs-generated layout are documented in Milestone 2 [`03_verifier_interface_and_wrappers.md`](../milestone-2/03_verifier_interface_and_wrappers.md).

## Repo Evidence

- Wrapper behavior (generated per blueprint; see root `.gitignore` for `ZKEmailVerifier.sol`):
  - [`circom/templates/ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera)
- Integrator-facing API (committed):
  - [`circom/contracts/src/interfaces/IZKEmailVerifier.sol`](../../contracts/src/interfaces/IZKEmailVerifier.sol)
- Signal sizing and template rendering:
  - [`circom/src/contract.rs`](../../src/contract.rs) (`prepare_contract_data`, `create_zkemail_verifier_*`, `generate_verifier_contract`)
- Full pipeline ordering (proof system + deploy):
  - [`circom/src/main.rs`](../../src/main.rs)

## Related documentation

- Milestone 2 [`03_verifier_interface_and_wrappers.md`](../milestone-2/03_verifier_interface_and_wrappers.md): verification flow and scope.
- Milestone 2 [`04_template_and_tooling.md`](../milestone-2/04_template_and_tooling.md): Tera templates and `snarkjs` verifier export.

## Demonstration

From the **repository root**:

```bash
cargo test -p circom prepare_contract_data
```

| Filter                  | Exercises                                                                                                                                               |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `prepare_contract_data` | [`prepare_contract_data`](../../src/contract.rs): public signal layout for the templated wrapper (counts for hashed regex vs non-hashed public parts). |

## Evidence standard for this deliverable

- Wrapper contract template encodes the expected proof layout and public-input indexing consistent with Groth16 + DKIM checks.
- Pipeline generates matching `Groth16Verifier` from the zkey for the same blueprint.

## Status

`Delivered`

Conclusion: Proof decoding, public-input conversion, and DKIM gating are implemented in the templated wrapper and fed by pipeline-generated contract data and verifier bytecode.
