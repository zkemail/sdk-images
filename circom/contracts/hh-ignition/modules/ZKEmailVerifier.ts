import { buildModule } from "@nomicfoundation/hardhat-ignition/modules";

const dkimRegistryEnv = process.env.DKIM_REGISTRY;

if (!dkimRegistryEnv) {
  throw new Error("DKIM_REGISTRY is not set");
}

export default buildModule("ZKEmailVerifierModule", (m) => {
  const dkimRegistry = m.getParameter("dkimRegistry", dkimRegistryEnv);

  const groth16Verifier = m.contract("Groth16Verifier");
  const zkEmailVerifier = m.contract("ZKEmailVerifier", [
    dkimRegistry,
    groth16Verifier,
  ]);

  return { groth16Verifier, zkEmailVerifier };
});
