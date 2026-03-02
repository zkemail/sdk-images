// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

import { Script, console } from "forge-std/Script.sol";
import { DKIMRegistry } from "@zk-email/contracts/DKIMRegistry.sol";

contract DeployDKIMRegistryScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        if (deployerPrivateKey == 0) {
            console.log("PRIVATE_KEY not set");
            return;
        }

        vm.startBroadcast(deployerPrivateKey);

        console.log("\n=== Step 0: Deploy DKIMRegistry ===");
        DKIMRegistry dkimRegistry = new DKIMRegistry(msg.sender);
        console.log("DKIMRegistry deployed at:", address(dkimRegistry));

        vm.stopBroadcast();

        console.log("\n=== Deployment Complete ===");
        console.log("DKIM_REGISTRY:", address(dkimRegistry));
    }
}
