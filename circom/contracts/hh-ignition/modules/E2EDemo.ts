import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

const dkimRegistryEnv = process.env.DKIM_REGISTRY;

if (!dkimRegistryEnv) {
  throw new Error("DKIM_REGISTRY is not set");
}

// Deploys the testBlueprint fixture's contracts (src/e2e-demo, copied verbatim from
// test/fixtures/testBlueprint) as the milestone-2 E2E demo -- see
// circom/kusama-grant/milestone-2/06_e2e_demo.md. dkimRegistry is expected to be the real,
// already-deployed milestone-1 DKIMRegistry, not a fresh one.
export default buildModule("E2EDemoModule", (m) => {
  const dkimRegistry = m.getParameter("dkimRegistry", dkimRegistryEnv);

  const groth16Verifier = m.contract("TestBlueprintGroth16Verifier");
  const zkEmailVerifier = m.contract("TestBlueprintZKEmailVerifier", [
    dkimRegistry,
    groth16Verifier,
  ]);

  return { groth16Verifier, zkEmailVerifier };
});
