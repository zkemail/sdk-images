// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import { console } from "forge-std/console.sol";
import { Script } from "forge-std/Script.sol";
import { IDKIMRegistry } from "@zk-email/contracts/interfaces/IDKIMRegistry.sol";
import { DKIMRegistry } from "@zk-email/contracts/DKIMRegistry.sol";
import { ClientProofVerifier } from "./tmp/ClientProofVerifier.sol";
import { ServerProofVerifier } from "./tmp/ServerProofVerifier.sol";
import { Contract, IVerifier } from "./tmp/Contract.sol";

contract Deploy is Script {
    IDKIMRegistry private dkimRegistry;

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        if (deployerPrivateKey == 0) {
            console.log("PRIVATE_KEY env var not set");
            return;
        }

        vm.startBroadcast(deployerPrivateKey);
        address dkimRegistryAddr;
        try vm.envAddress("DKIM_REGISTRY") returns (address addr) {
            dkimRegistryAddr = addr;
        } catch {
            dkimRegistryAddr = address(0);
        }

        if (dkimRegistryAddr == address(0)) {
            dkimRegistry = new DKIMRegistry(msg.sender);
            dkimRegistryAddr = address(dkimRegistry);
        }

        dkimRegistry = IDKIMRegistry(dkimRegistryAddr);

        IVerifier cpv = new ClientProofVerifier();
        IVerifier spv = new ServerProofVerifier();
        Contract circuitContract = new Contract(dkimRegistry, cpv, spv);
        vm.stopBroadcast();

        console.log("Deployed Verifier at", address(verifier));
        console.log("Deployed Contract at", address(circuitContract));
        console.log("Deployed DKIMRegistry at", dkimRegistryAddr);
        return;
    }
}
