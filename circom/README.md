## Circom helper CLI

This crate includes a small helper CLI for working with the ZKEmail Circom pipeline without compiling a full circuit.

### Generate example Solidity contracts only

You can generate example Solidity contracts for a Foundry project (so it can compile and run tests) without running Circom or `snarkjs` by using:

```bash
cargo run -p circom -- generate-example-contracts ./example-contract-data.json ./contracts/src
```

- `./example-contract-data.json` JSON payload matching the `ContractData` schema used by the Circom tooling.
- `./contracts/src` output directory where:
  - `ZKEmailVerifier.sol` will be written.
  - `interfaces/IGroth16Verifier.sol` will be written.

This is especially useful for local Solidity-only development where you want contracts to compile and tests to run without doing a full proof/circuit build.
