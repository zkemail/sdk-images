// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.34;

import { Script, console } from "forge-std/Script.sol";
import { DKIMRegistryMock } from "../test/DKIMRegistryMock.sol";

contract DeployMockDKIMRegistryScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        if (deployerPrivateKey == 0) {
            console.log("PRIVATE_KEY not set");
            return;
        }

        vm.startBroadcast(deployerPrivateKey);

        console.log("\n=== Step 0: Deploy MockDKIMRegistry ===");
        DKIMRegistryMock mockDkimRegistry = new DKIMRegistryMock();
        console.log("DKIMRegistryMock deployed at:", address(mockDkimRegistry));

        vm.stopBroadcast();

        console.log("\n=== Deployment Complete ===");
        console.log("DKIM_REGISTRY_MOCK:", address(mockDkimRegistry));
    }
}
