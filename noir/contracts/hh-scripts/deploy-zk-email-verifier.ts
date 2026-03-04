import { ethers, network } from "hardhat";
import { vars } from "hardhat/config";

async function main() {
  // Resolve DKIM registry address from Hardhat vars or environment
  let dkimRegistryAddr: string | undefined;
  try {
    dkimRegistryAddr = vars.get("DKIM_REGISTRY");
  } catch {
    dkimRegistryAddr = process.env.DKIM_REGISTRY;
  }

  if (!dkimRegistryAddr) {
    console.error("DKIM_REGISTRY not set (Hardhat vars or environment)");
    return;
  }

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
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
