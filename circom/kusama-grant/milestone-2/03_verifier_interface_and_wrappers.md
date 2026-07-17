# 03 - Verifier Interface and Wrappers

Milestone 2 on-chain integration surface: a generic `IZKEmailVerifier` interface plus the per-blueprint Groth16 + DKIM wrapper that checks and formats a proof before verifying it.

## Implementation Notes

- **Entry API:** Integrators target [`IZKEmailVerifier`](../../contracts/src/interfaces/IZKEmailVerifier.sol): a single `verify(bytes calldata proof, bytes32[] calldata publicInputs)` method and typed `error`s for failure cases (invalid registry/verifier, wrong public-input length, invalid DKIM key hash, invalid proof).
- **Wrapper:** Each blueprint emits `ZKEmailVerifier.sol` from [`circom/templates/ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera) (via [`circom/src/contract.rs`](../../src/contract.rs)). That contract composes:
  - [`IDKIMRegistry`](../../contracts/src/interfaces/IDKIMRegistry.sol): `isKeyHashValid(domainHash, keyHash)` before accepting the proof’s public key hash.
  - `IGroth16Verifier` / `Groth16Verifier`: generated alongside the blueprint’s proving key; `verifyProof` takes decoded Groth16 `(pA, pB, pC)` and a fixed-size public-signals array aligned with the circuit.
- **Proof preparation in-contract:** `verify` checks public-input count, validates the DKIM key hash at a fixed offset in `publicInputs`, `abi.decode`s the calldata proof into Groth16 points, copies `bytes32[]` into `uint256[]` for the underlying verifier, then calls `verifyProof`.
- **What is committed vs generated:** Stable, repo-tracked artifacts are the integrator-facing [`IZKEmailVerifier`](../../contracts/src/interfaces/IZKEmailVerifier.sol) and [`IDKIMRegistry`](../../contracts/src/interfaces/IDKIMRegistry.sol). Per-blueprint outputs (`ZKEmailVerifier.sol`, `Groth16Verifier.sol`, and `IGroth16Verifier.sol` under [`circom/contracts/src/`](../../contracts/src/)) are **not** committed (see repository root [`.gitignore`](../../../.gitignore)); they are produced when a blueprint is built and exist only in working trees / distribution zips, not as canonical source in git.
- **Tests:** [`test/DKIMRegistryMock.sol`](../../contracts/test/DKIMRegistryMock.sol) implements `IDKIMRegistry` for local or Foundry-style tests.

## Verification flow (high level)

1. Require `publicInputs.length == PUBLIC_INPUTS_LENGTH`.
2. Call `DKIM_REGISTRY.isKeyHashValid(DOMAIN_HASH, publicInputs[PUBLIC_KEY_HASH_OFFSET])`; revert if false.
3. Decode `proof` as `(uint256[2] pA, uint256[2][2] pB, uint256[2] pC)`.
4. Build `uint256[PUBLIC_INPUTS_LENGTH] pubSignals` from `publicInputs`.
5. Require `GROTH16_VERIFIER.verifyProof(pA, pB, pC, pubSignals)`.

## Repo Evidence

- Committed interfaces (integration surface + DKIM registry API):
  - [`circom/contracts/src/interfaces/IZKEmailVerifier.sol`](../../contracts/src/interfaces/IZKEmailVerifier.sol)
  - [`circom/contracts/src/interfaces/IDKIMRegistry.sol`](../../contracts/src/interfaces/IDKIMRegistry.sol)
- Generated per blueprint (not tracked in git; paths are where the pipeline writes them for compile/deploy):
  - `circom/contracts/src/ZKEmailVerifier.sol`
  - `circom/contracts/src/Groth16Verifier.sol`
  - `circom/contracts/src/interfaces/IGroth16Verifier.sol`
- Template / generator (source of truth for wrapper + Groth16 surface):
  - [`circom/templates/ZKEmailVerifier.sol.tera`](../../templates/ZKEmailVerifier.sol.tera)
  - [`circom/templates/IGroth16Verifier.sol.tera`](../../templates/IGroth16Verifier.sol.tera)
  - [`circom/templates/MockGroth16Verifier.sol.tera`](../../templates/MockGroth16Verifier.sol.tera) (used to emit a mock `Groth16Verifier.sol` in some flows; production verifiers come from the proving pipeline)
  - [`circom/src/contract.rs`](../../src/contract.rs)
- DKIM registry mock (testing):
  - [`circom/contracts/test/DKIMRegistryMock.sol`](../../contracts/test/DKIMRegistryMock.sol)

## Scope note

- This milestone ships **Groth16** as the concrete proof backend in generated contracts (`IGroth16Verifier`, `Groth16Verifier`, `ZKEmailVerifier` per blueprint). Other proof systems are not implemented; new backends can follow the same composition pattern (registry + proof verifier + shared `IZKEmailVerifier` shape) without changing how DKIM is checked.

## Related documentation

- Contracts package overview (DKIM registry sourcing, env vars): [`circom/contracts/README.md`](../../contracts/README.md)
- Local deploy evidence (wrapper deployed via Ignition): [`02_local_environment.md`](./02_local_environment.md)
- Templates and deploy tooling: [`04_template_and_tooling.md`](./04_template_and_tooling.md)
