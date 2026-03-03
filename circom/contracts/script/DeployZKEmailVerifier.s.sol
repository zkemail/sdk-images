// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

import { Script, console } from "forge-std/Script.sol";
import { IDKIMRegistry } from "../src/interfaces/IDKIMRegistry.sol";
import { Groth16Verifier } from "../src/Groth16Verifier.sol";
import { IGroth16Verifier, ZKEmailVerifier } from "../src/ZKEmailVerifier.sol";

contract DeployZKEmailVerifierScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        if (deployerPrivateKey == 0) {
            console.log("PRIVATE_KEY not set");
            return;
        }

        address dkimRegistryAddr = vm.envAddress("DKIM_REGISTRY");
        if (dkimRegistryAddr == address(0)) {
            console.log("DKIM_REGISTRY not set");
            return;
        }
        IDKIMRegistry dkimRegistry = IDKIMRegistry(dkimRegistryAddr);

        vm.startBroadcast(deployerPrivateKey);

        console.log("\n=== Step 0: Deploy Groth16Verifier ===");
        Groth16Verifier groth16Verifier = new Groth16Verifier();
        console.log("Groth16Verifier deployed at:", address(groth16Verifier));

        console.log("\n=== Step 1: Deploy ZKEmailVerifier ===");
        console.log("Deploying ZKEmailVerifier with DKIMRegistry:", address(dkimRegistry));
        ZKEmailVerifier zkEmailVerifier =
            new ZKEmailVerifier(dkimRegistry, IGroth16Verifier(address(groth16Verifier)));
        console.log("ZKEmailVerifier deployed at:", address(zkEmailVerifier));

        vm.stopBroadcast();

        console.log("\n=== Deployment Complete ===");
        console.log("GROTH16_VERIFIER:", address(groth16Verifier));
        console.log("ZK_EMAIL_VERIFIER:", address(zkEmailVerifier));
    }
}
