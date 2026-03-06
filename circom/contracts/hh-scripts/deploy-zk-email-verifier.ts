import fs from "fs";
import path from "path";
import { ethers, network } from "hardhat";
import { requireEnv } from "../hh-utils/require-env";

const DEPLOYMENTS_DIR = "hh-deployments";

const main = async () => {
  const dkimRegistryAddr = requireEnv("DKIM_REGISTRY");

  if (dkimRegistryAddr === ethers.ZeroAddress) {
    throw new Error("DKIM_REGISTRY is the zero address");
  }

  const [deployer] = await ethers.getSigners();
  if (!deployer) {
    throw new Error("No deployer signer available. Ensure PRIVATE_KEY is set");
  }

  console.log(`\nUsing network: ${network.name}`);
  console.log(`Deployer address: ${await deployer.getAddress()}`);
  console.log(`DKIM_REGISTRY: ${dkimRegistryAddr}`);

  console.log("\n=== Step 0: Deploy Groth16Verifier ===");
  const Groth16VerifierFactory = await ethers.getContractFactory(
    "Groth16Verifier",
    deployer,
  );
  const groth16Verifier = await Groth16VerifierFactory.deploy();
  await groth16Verifier.waitForDeployment();
  const groth16VerifierAddress = await groth16Verifier.getAddress();
  console.log("Groth16Verifier deployed at:", groth16VerifierAddress);

  console.log("\n=== Step 1: Deploy ZKEmailVerifier ===");
  console.log("Deploying ZKEmailVerifier with DKIMRegistry:", dkimRegistryAddr);
  const ZKEmailVerifierFactory = await ethers.getContractFactory(
    "ZKEmailVerifier",
    deployer,
  );
  const zkEmailVerifier = await ZKEmailVerifierFactory.deploy(
    dkimRegistryAddr,
    groth16VerifierAddress,
  );
  await zkEmailVerifier.waitForDeployment();
  const zkEmailVerifierAddress = await zkEmailVerifier.getAddress();
  console.log("ZKEmailVerifier deployed at:", zkEmailVerifierAddress);

  console.log("\n=== Deployment Complete ===");
  console.log("GROTH16_VERIFIER:", groth16VerifierAddress);
  console.log("ZK_EMAIL_VERIFIER:", zkEmailVerifierAddress);

  const chainId = (await ethers.provider.getNetwork()).chainId;
  const deploymentsDir = path.join(DEPLOYMENTS_DIR, chainId.toString());
  const deploymentsFile = path.join(deploymentsDir, "run-latest.json");
  fs.mkdirSync(deploymentsDir, { recursive: true });
  fs.writeFileSync(
    deploymentsFile,
    JSON.stringify(
      {
        GROTH16_VERIFIER: groth16VerifierAddress,
        ZK_EMAIL_VERIFIER: zkEmailVerifierAddress,
      },
      null,
      2,
    ),
  );
  console.log(`Deployment addresses saved to ${deploymentsFile}`);
};

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
