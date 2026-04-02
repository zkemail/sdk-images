# 05 - Documentation

Deliverable mapping: Milestone 2, Deliverable 5 (`Documentation`).

## What must be delivered

- Public documentation with usage instructions.

## Implementation Notes

Documentation for this milestone spans **three layers**:

1. **Contracts package (operators / integrators)** — [`circom/contracts/README.md`](../../contracts/README.md)  
   Describes package layout, DKIM registry sourcing (external zk.email docs and `@zk-email/contracts`), environment variables, Hardhat Ignition deploy and verify and command reference. Network examples include `localPvm`, `localEvm`, Base Sepolia (`84532`), Polkadot Hub testnet (`420420417`), and Ethereum Sepolia (`11155111`). Numeric networks read `RPC_URL` / `PRIVATE_KEY` from the environment when set (see [`contracts/README.md`](../../contracts/README.md)). For **which** variables apply to which local network profile, use [`02_local_environment.md`](./02_local_environment.md) together with [`hardhat.config.ts`](../../contracts/hardhat.config.ts).

2. **Circom crate (pipeline / local Solidity-only)** — [`circom/README.md`](../../README.md)  
   Documents `generate-example-contracts` for populating gitignored `src/*.sol` from [`example-contract-data.json`](../../example-contract-data.json) without a full circuit build. This complements the contracts README: a fresh clone needs generated verifiers before `yarn build` (see [`02_local_environment.md`](./02_local_environment.md)).

3. **Grant evidence and structure (this directory)** — [`circom/kusama-grant/README.md`](../README.md) indexes milestones; Milestone 2 detail lives under [`milestone-2/`](./):

   | Doc                                                                                | Role                                                                        |
   | ---------------------------------------------------------------------------------- | --------------------------------------------------------------------------- |
   | [`00_overview.md`](./00_overview.md)                                               | Milestone goal, deliverable table, current status, summary                  |
   | [`01_project_setup.md`](./01_project_setup.md)                                     | Hardhat + Foundry + resolc, responsibilities, verification limits           |
   | [`02_local_environment.md`](./02_local_environment.md)                             | Generate contracts → local PVM + local EVM flows, `.env`, commands, outputs |
   | [`03_verifier_interface_and_wrappers.md`](./03_verifier_interface_and_wrappers.md) | Interfaces vs generated wrapper, verification flow                          |
   | [`04_template_and_tooling.md`](./04_template_and_tooling.md)                       | Tera templates, bundle, deploy/verify tooling                               |
   | [`05_documentation_and_howto.md`](./05_documentation_and_howto.md)                 | This map of public and grant-facing docs                                    |

## Repo Evidence

- Operator-facing how-to:
  - [`circom/contracts/README.md`](../../contracts/README.md)
- Circom CLI / example generation:
  - [`circom/README.md`](../../README.md)
  - [`circom/example-contract-data.json`](../../example-contract-data.json)
- Grant index and Milestone 2 narrative:
  - [`circom/kusama-grant/README.md`](../README.md)
  - [`circom/kusama-grant/milestone-2/00_overview.md`](./00_overview.md)
  - Deliverables [`01`](./01_project_setup.md)–[`04`](./04_template_and_tooling.md) as linked above

## Related documentation

- End-to-end local proof commands: [`02_local_environment.md`](./02_local_environment.md)
- Template and deploy/verify tooling detail: [`04_template_and_tooling.md`](./04_template_and_tooling.md)

## Evidence standard for this deliverable

- Public READMEs exist under `circom/contracts` and `circom` with actionable commands and environment expectations.
- Grant documentation provides a traceable index and per-deliverable evidence that aligns with those READMEs and with reproducible local flows.

## Status

`Delivered`

Conclusion: Usage documentation is publicly available in the contracts and Circom package READMEs; grant milestone docs under `kusama-grant/milestone-2` index and elaborate delivery evidence, including local dual-environment instructions in deliverable 02.
