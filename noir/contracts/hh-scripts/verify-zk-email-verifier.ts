import fs from "fs";
import path from "path";
import { ethers } from "hardhat";
import { requireEnv } from "../utils/require-env";
import { verifyWithRetry } from "../utils/verify-with-retry";

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

  const honkVerifierAddr =
    process.env.HONK_VERIFIER || deployments.HONK_VERIFIER;
  const zkEmailVerifierAddr =
    process.env.ZK_EMAIL_VERIFIER || deployments.ZK_EMAIL_VERIFIER;

  if (!honkVerifierAddr || !zkEmailVerifierAddr) {
    console.error(
      "Error: Could not determine deployed addresses. " +
        "Set HONK_VERIFIER and ZK_EMAIL_VERIFIER in env " +
        `or run the deploy script first to create ${deploymentsFile}.`,
    );
    process.exitCode = 1;
    return;
  }

  console.log(`\nUsing chain: ${chainId}`);
  console.log(`HONK_VERIFIER: ${honkVerifierAddr}`);
  console.log(`ZK_EMAIL_VERIFIER: ${zkEmailVerifierAddr}`);
  console.log(`DKIM_REGISTRY: ${dkimRegistryAddr}`);

  console.log("\n=== Verifying HonkVerifier ===");
  await verifyWithRetry("HonkVerifier", {
    address: honkVerifierAddr,
    constructorArguments: [],
  });

  console.log("\n=== Verifying ZKEmailVerifier ===");
  await verifyWithRetry("ZKEmailVerifier", {
    address: zkEmailVerifierAddr,
    constructorArguments: [dkimRegistryAddr, honkVerifierAddr],
  });

  console.log("\n=== All contracts verified ===");
};

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
