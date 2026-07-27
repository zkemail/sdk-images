# e2e-demo

Verbatim copies of the committed test fixture at
[`test/fixtures/testBlueprint/`](../../test/fixtures/testBlueprint/) (only relative import paths
changed, no logic differs), placed under `src/` so Hardhat can compile and deploy them -- Hardhat
only compiles from `src/`, not `test/`.

Deployed to Paseo (chain `420420417`), wired to the real milestone-1 `DKIMRegistry`
(`0xD9e492f8104Ec730AF47A1A5C0cEAf94C89Da8EE`) rather than a fresh or mock registry:

| Contract | Address |
| --- | --- |
| `TestBlueprintGroth16Verifier` | [`0x2B1D8681B837a9e9080D36E9b588D1275D8e5D04`](https://blockscout-testnet.polkadot.io/address/0x2B1D8681B837a9e9080D36E9b588D1275D8e5D04) |
| `TestBlueprintZKEmailVerifier` | [`0x68E81c9909aD3982A991a953A63729bbF906B72B`](https://blockscout-testnet.polkadot.io/address/0x68E81c9909aD3982A991a953A63729bbF906B72B) |

Deployed via [`hh-ignition/modules/E2EDemo.ts`](../../hh-ignition/modules/E2EDemo.ts). Full
deployment record (bytecode provenance, the real proof's `verify()` transaction, reproduce
commands) is in `kusama-grant/milestone-2/06_e2e_demo.md`.
