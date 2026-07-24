# 03 - Verifier Interface and Wrappers

Milestone 2 on-chain integration surface: a generic `IZKEmailVerifier` interface plus the per-blueprint Groth16 + DKIM wrapper that checks and formats a proof before verifying it.

## Implementation Notes

- **Entry API:** Integrators target [`IZKEmailVerifier`](../../contracts/src/interfaces/IZKEmailVerifier.sol): a single `verify(bytes calldata proof, bytes32[] calldata publicInputs)` method and typed `error`s for failure cases (invalid registry/verifier, wrong public-input length, invalid DKIM key hash, invalid proof).
- **Wrapper:** Each blueprint emits `ZKEmailVerifier.sol` from [`circom/templates/ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera) (via [`circom/src/contract.rs`](../../src/contract.rs)). That contract composes:
  - [`IDKIMRegistry`](../../contracts/src/interfaces/IDKIMRegistry.sol): `isKeyHashValid(domainHash, keyHash)` before accepting the proof’s public key hash.
  - `IGroth16Verifier` / `Groth16Verifier`: generated alongside the blueprint’s proving key; `verifyProof` takes decoded Groth16 `(pA, pB, pC)` and a fixed-size public-signals array aligned with the circuit.
- **Proof preparation in-contract:** `verify` checks public-input count, validates the DKIM key hash at a fixed offset in `publicInputs`, `abi.decode`s the calldata proof into Groth16 points, copies `bytes32[]` into `uint256[]` for the underlying verifier, then calls `verifyProof`.
- **What is committed vs generated:** Stable, repo-tracked artifacts are the integrator-facing [`IZKEmailVerifier`](../../contracts/src/interfaces/IZKEmailVerifier.sol) and [`IDKIMRegistry`](../../contracts/src/interfaces/IDKIMRegistry.sol). Per-blueprint outputs (`ZKEmailVerifier.sol`, `Groth16Verifier.sol`, and `IGroth16Verifier.sol` under [`circom/contracts/src/`](../../contracts/src/)) are **not** committed (see repository root [`.gitignore`](../../../.gitignore)); they are produced when a blueprint is built and exist only in working trees / distribution zips, not as canonical source in git. This is intentional, not an evidence gap: this milestone's deliverable is the *tooling* that renders and wires a wrapper for whatever circuit a blueprint compiles, not any specific circuit's on-chain verification, and there is no testnet-deployment line item for this milestone (unlike milestone 1's DKIM registry). To make the generated shape reviewable anyway, run the same generator the tooling uses:
  ```bash
  cargo run -p circom -- generate-example-contracts ./example-contract-data.json ./contracts/src
  ```
  (from [`circom/`](../../); see [`05_public_howto.md`](./05_public_howto.md) step 2).
- **Tests:** [`test/DKIMRegistryMock.sol`](../../contracts/test/DKIMRegistryMock.sol) implements `IDKIMRegistry` for local or Foundry-style tests. [`test/TestBlueprintZKEmailVerifier.t.sol`](../../contracts/test/TestBlueprintZKEmailVerifier.t.sol) and its fixtures under [`test/fixtures/testBlueprint/`](../../contracts/test/fixtures/testBlueprint/) go further: they commit a concrete, reviewable `ZKEmailVerifier` instance (`TestBlueprintZKEmailVerifier.sol`) and a real (non-mock) Groth16 verifier (`TestBlueprintGroth16Verifier.sol`) — both copied verbatim from a real blueprint's own generated output — and run a real Groth16 proof (obtained via real remote proving against a real email) through `verify()` end-to-end, proving the proof-decode/DKIM-gate plumbing works against real verification math, not `MockGroth16Verifier`'s always-`true` stub. See [`test/fixtures/testBlueprint/README.md`](../../contracts/test/fixtures/testBlueprint/README.md) for full provenance and what this fixture does and doesn't prove. This now runs in CI on every push (`.github/workflows/circom-contracts-tests.yml`). Example passing `run_contracts_tests` job (2026-07-24): https://github.com/zkemail/sdk-images/actions/runs/30117616346/job/89562063336. For the current state of the branch, see the [Actions tab](https://github.com/zkemail/sdk-images/actions/workflows/circom-contracts-tests.yml?query=branch%3Astaging).
- **Registry compatibility:** the wrapper only depends on [`IDKIMRegistry.isKeyHashValid`](../../contracts/src/interfaces/IDKIMRegistry.sol) at the interface level, so it works unmodified with any conforming registry — both the plain `DKIMRegistry` deployed for milestone 1 and `UserOverrideableDKIMRegistry` implement the same `IERC7969`/`IDKIMRegistry` interface. Whichever registry address is passed into the wrapper's constructor is the production target; there is no registry-specific code in the wrapper.

## Verification flow (high level)

1. Require `publicInputs.length == PUBLIC_INPUTS_LENGTH`.
2. Call `DKIM_REGISTRY.isKeyHashValid(DOMAIN_HASH, publicInputs[PUBLIC_KEY_HASH_OFFSET])`; revert if false.
3. Decode `proof` as `(uint256[2] pA, uint256[2][2] pB, uint256[2] pC)`.
4. Build `uint256[PUBLIC_INPUTS_LENGTH] pubSignals` from `publicInputs`.
5. Require `GROTH16_VERIFIER.verifyProof(pA, pB, pC, pubSignals)`.

## Scope note

- This milestone ships **Groth16** as the concrete proof backend in generated contracts (`IGroth16Verifier`, `Groth16Verifier`, `ZKEmailVerifier` per blueprint). Other proof systems are not implemented; new backends can follow the same composition pattern (registry + proof verifier + shared `IZKEmailVerifier` shape) without changing how DKIM is checked.
