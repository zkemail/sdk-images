// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {ZKEmailProof} from "../src/ZKEmailProof.sol";

contract ZKEmailProof_Script is Script {
    function run() public {
        vm.startBroadcast(vm.envUint("PRIVATE_KEY"));

        address owner = vm.envAddress("INITIAL_OWNER");
        new ZKEmailProof(owner);

        // TODO: (merge-ok) Set verifier

        vm.stopBroadcast();
    }
}
