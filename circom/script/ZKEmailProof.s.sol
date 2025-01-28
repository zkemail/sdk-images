// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {DKIMRegistry} from "@zk-email/contracts/DKIMRegistry.sol";
import {IVerifier} from "../contracts/interfaces/IVerifier.sol";
import {ZKEmailProof, Proof, ZKEmailProofMetadata} from "../contracts/ZKEmailProof.sol";
import {TestVerifier} from "../contracts/test/TestVerifier.sol";

contract ZKEmailProof_Script is Script {
    address constant DEPLOYED_VERIFIER =
        0x7019c2E274c77dd6E9e4C2707068BC6e690eA0AF;

    DKIMRegistry dkimRegistry;
    IVerifier groth16Verifier;
    ZKEmailProof zkEmailProof;
    TestVerifier verifier;

    string domainName;
    bytes32 publicKeyHash;

    function run() public {
        vm.startBroadcast(vm.envUint("PRIVATE_KEY"));

        // Assume owner is deployer for testing
        address owner = vm.envAddress("OWNER");

        dkimRegistry = new DKIMRegistry(owner);
        groth16Verifier = IVerifier(DEPLOYED_VERIFIER);
        zkEmailProof = new ZKEmailProof(owner);
        verifier = new TestVerifier(
            address(dkimRegistry),
            address(groth16Verifier),
            address(zkEmailProof)
        );

        domainName = "accounts.google.com";
        publicKeyHash = bytes32(
            uint256(
                3024598485745563149860456768272954250618223591034926533254923041921841324429
            )
        );

        zkEmailProof.addVerifier(address(verifier));
        dkimRegistry.setDKIMPublicKeyHash(domainName, publicKeyHash);

        vm.stopBroadcast();
    }
}
