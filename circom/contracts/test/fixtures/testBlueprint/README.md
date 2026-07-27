# testBlueprint fixture

A real (non-mock) Groth16 proof, its real generated verifier, and the real generated `ZKEmailVerifier`
wrapper -- all produced by *this repo's own tooling* for a real test blueprint -- used by
`TestBlueprintZKEmailVerifierTest` to prove `ZKEmailVerifier.verify()`'s proof-decode/DKIM-gate
plumbing works end-to-end against real Groth16 verification math, not `MockGroth16Verifier`'s
always-`true` stub.

## Provenance

- **`parameters.json`** is this blueprint's configuration (sender domain `x.com`,
  `ignore_body_hash_check: true`, the single `email_sender` decomposed regex, etc.) -- the same
  `ContractData`-relevant inputs this repo's own tooling takes to render a wrapper.
- **`TestBlueprintZKEmailVerifier.sol`** and **`TestBlueprintGroth16Verifier.sol`** are copied
  verbatim (only contract names, and the Groth16 verifier interface type, changed) from that
  blueprint's own generated `contracts/src/ZKEmailVerifier.sol` / `contracts/src/Groth16Verifier.sol`
  -- i.e. this is *this repository's own generator's real output* for a real blueprint, not a
  synthetic or hand-built example.
- **`TestBlueprintFixture.sol`**'s proof/public-input values come from a real remote-proving request
  for a real `x.com` email (DKIM `d=x.com`, verified `dkim=pass` by Google at delivery time).
  **`proof.json`** / **`public.json`** are the raw Groth16 proof and decoded public output
  (`email_sender: ["info@x.com"]`) as returned by the prover, before the G2-coordinate swap and
  hex/`bytes32` transcription into `TestBlueprintFixture.sol`'s Solidity constants -- so the committed
  Solidity values can be independently checked against this raw data. `email_sender` being correctly
  decoded confirms the proof is real and the circuit actually extracted the sender address from the
  email, rather than being fabricated.

This fixture is a deliberately static, pre-computed snapshot, not a live pipeline: the proof was
generated once and the result committed, rather than the test regenerating it from source (an email,
through witness generation and proving) on every run. That's intentional here, not a shortcut --
regenerating at test/CI time would mean depending on the registry's live proving infrastructure (which
this session watched go down mid-conversation from a preempted GKE Spot node) and on the source
email's DKIM key never rotating, and it would reintroduce the full circuit/proving pipeline this
milestone's tooling doesn't need to demonstrate (that's milestone 3's scope). A static fixture is fast,
deterministic, and has no external dependencies at test time -- at the cost of not catching drift if
the wrapper template or generation logic changes; that tradeoff is worth it here.

This fixture is scoped to milestone 2 (verifier contract tooling): it demonstrates the wrapper/verifier
generation and proof-decode plumbing, not the registry/remote-proving/deployment pipeline itself,
which is out of scope here.

## What this is not

This is a test fixture, not a per-blueprint generated artifact. The two `.sol` files here are
distinctly named (and not gitignored) specifically so they're never confused with the real,
per-blueprint `ZKEmailVerifier.sol` / `Groth16Verifier.sol` / `IGroth16Verifier.sol` that
`cargo run -p circom -- generate-example-contracts ...` (or the full production pipeline) writes into
`contracts/src/` -- those remain gitignored, generated on demand; see
`../../../../kusama-grant/milestone-2/03_verifier_interface_and_wrappers.md` for how to generate and
inspect one yourself.

`TestBlueprintZKEmailVerifier`'s `PUBLIC_KEY_HASH_OFFSET = 0` is correct for this specific circuit (no
header/body masking enabled) -- see the milestone docs for when that offset shifts. The DKIM registry
check itself is mocked (`DKIMRegistryMock`) in the test, deliberately: this fixture's job is proving the
proof-decode/dispatch plumbing against real Groth16 verification math, not re-exercising
`DKIMRegistry`'s own logic, which already has its own dedicated test suite in `zk-email-verify`.
Interface compatibility with a real, live-deployed `DKIMRegistry` is demonstrated separately,
end-to-end, on Paseo, using this exact fixture deployed from
[`src/e2e-demo/`](../../../src/e2e-demo/) -- see that directory's README and
`kusama-grant/milestone-2/06_e2e_demo.md` for the deployment record.
