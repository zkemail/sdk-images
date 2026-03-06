import fs from "fs";
import path from "path";
import { ethers, network } from "hardhat";
import { requireEnv } from "../utils/requireEnv";

const DEPLOYMENTS_DIR = "hh-deployments";

const main = async () => {
  const dkimRegistryAddr = requireEnv("DKIM_REGISTRY");

  if (dkimRegistryAddr === ethers.ZeroAddress) {
    console.error("DKIM_REGISTRY is the zero address");
    return;
  }

  const [deployer] = await ethers.getSigners();
  if (!deployer) {
    console.error("No deployer signer available");
    return;
  }

  console.log(`\nUsing network: ${network.name}`);
  console.log(`Deployer address: ${await deployer.getAddress()}`);
  console.log(`DKIM_REGISTRY: ${dkimRegistryAddr}`);

  console.log("\n=== Step 0: Deploy HonkVerifier ===");
  const HonkVerifierFactory = await ethers.getContractFactory(
    "HonkVerifier",
    deployer,
  );
  const honkVerifier = await HonkVerifierFactory.deploy();
  await honkVerifier.waitForDeployment();
  const honkVerifierAddress = await honkVerifier.getAddress();
  console.log("HonkVerifier deployed at:", honkVerifierAddress);

  console.log("\n=== Step 1: Deploy ZKEmailVerifier ===");
  console.log("Deploying ZKEmailVerifier with DKIMRegistry:", dkimRegistryAddr);
  const ZKEmailVerifierFactory = await ethers.getContractFactory(
    "ZKEmailVerifier",
    deployer,
  );
  const zkEmailVerifier = await ZKEmailVerifierFactory.deploy(
    dkimRegistryAddr,
    honkVerifierAddress,
  );
  await zkEmailVerifier.waitForDeployment();
  const zkEmailVerifierAddress = await zkEmailVerifier.getAddress();
  console.log("ZKEmailVerifier deployed at:", zkEmailVerifierAddress);

  console.log("\n=== Deployment Complete ===");
  console.log("HONK_VERIFIER:", honkVerifierAddress);
  console.log("ZK_EMAIL_VERIFIER:", zkEmailVerifierAddress);

  const chainId = (await ethers.provider.getNetwork()).chainId;
  const deploymentsDir = path.join(DEPLOYMENTS_DIR, chainId.toString());
  const deploymentsFile = path.join(deploymentsDir, "run-latest.json");
  fs.mkdirSync(deploymentsDir, { recursive: true });
  fs.writeFileSync(
    deploymentsFile,
    JSON.stringify(
      {
        HONK_VERIFIER: honkVerifierAddress,
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
