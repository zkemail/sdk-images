## Noir helper CLI

This crate includes a small helper CLI for working with the ZKEmail Noir pipeline without compiling a full circuit.

### Generate example Solidity contracts only

You can generate example Solidity contracts for a Foundry project (so it can compile and run tests) without running `nargo` or `bb` by using:

```bash
cargo run -p noir -- generate-example-contracts ./example-contract-data.json ./contracts/src
```

- `./example-contract-data.json` – JSON payload matching the `ExampleContractData` schema used by the Noir tooling.
- `./contracts/src` – (optional) output directory. If omitted, files are written to `noir/contracts/src/`.

The command writes:

- `HonkVerifier.sol` – mock Honk verifier (test-only; always returns configurable validity).
- `ZKEmailVerifier.sol` – ZKEmail verifier rendered with the given sender domain and public inputs length.

Example JSON:

```json
{
  "senderDomain": "example.com",
  "publicInputsLength": 2
}
```

This is especially useful for local Solidity-only development where you want contracts to compile and `yarn build` to pass without doing a full circuit compile.
