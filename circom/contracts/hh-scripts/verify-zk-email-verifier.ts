import fs from "fs";
import path from "path";
import hre, { ethers } from "hardhat";
import { requireEnv } from "../hh-utils/require-env";

const DEPLOYMENTS_DIR = "hh-deployments";

const main = async () => {
  const dkimRegistryAddr = requireEnv("DKIM_REGISTRY");

  const chainId = (await ethers.provider.getNetwork()).chainId;
  const deploymentsFile = path.join(
    DEPLOYMENTS_DIR,
    chainId.toString(),
    "run-latest.json",
  );
  let deployments: Record<string, string> = {};

  if (fs.existsSync(deploymentsFile)) {
    console.log(`Reading deployed addresses from ${deploymentsFile}`);
    deployments = JSON.parse(fs.readFileSync(deploymentsFile, "utf-8"));
  }

  const groth16VerifierAddr =
    process.env.GROTH16_VERIFIER || deployments.GROTH16_VERIFIER;
  const zkEmailVerifierAddr =
    process.env.ZK_EMAIL_VERIFIER || deployments.ZK_EMAIL_VERIFIER;

  if (!groth16VerifierAddr || !zkEmailVerifierAddr) {
    console.error(
      "Error: Could not determine deployed addresses. " +
        "Set GROTH16_VERIFIER and ZK_EMAIL_VERIFIER in env " +
        `or run the deploy script first to create ${deploymentsFile}.`,
    );
    process.exitCode = 1;
    return;
  }

  console.log(`\nUsing chain: ${chainId}`);
  console.log(`GROTH16_VERIFIER: ${groth16VerifierAddr}`);
  console.log(`ZK_EMAIL_VERIFIER: ${zkEmailVerifierAddr}`);
  console.log(`DKIM_REGISTRY: ${dkimRegistryAddr}`);

  console.log("\n=== Verifying Groth16Verifier ===");
  await hre.run("verify:verify", {
    address: groth16VerifierAddr,
    constructorArguments: [],
  });

  console.log("\n=== Verifying ZKEmailVerifier ===");
  await hre.run("verify:verify", {
    address: zkEmailVerifierAddr,
    constructorArguments: [dkimRegistryAddr, groth16VerifierAddr],
  });

  console.log("\n=== All contracts verified ===");
};

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
