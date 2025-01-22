// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {DKIMRegistry} from "@zk-email/contracts/DKIMRegistry.sol";
import {ZKEmailProof, Proof} from "../../../contracts/ZKEmailProof.sol";
import {MockGroth16Verifier} from "../../../contracts/test/MockGroth16Verifier.sol";

import {TestVerifier} from "../../../contracts/test/TestVerifier.sol";

contract BaseTest is Test {
    DKIMRegistry dkimRegistry;
    MockGroth16Verifier groth16Verifier;
    ZKEmailProof zkEmailProof;
    TestVerifier verifier;

    address public owner;
    address public alice;
    address public bob;

    uint256 blueprintId;
    Proof proof;
    uint256[] publicOutputs = new uint256[](1);
    string decodedPublicOutputs;
    uint256 proverEthAddressIdx;

    string domainName;
    bytes32 publicKeyHash;

    function setUp() public virtual {
        owner = address(1);
        alice = address(2);
        bob = address(3);

        dkimRegistry = new DKIMRegistry(owner);
        groth16Verifier = new MockGroth16Verifier();
        zkEmailProof = new ZKEmailProof(owner);

        verifier = new TestVerifier(
            address(dkimRegistry),
            address(groth16Verifier),
            address(zkEmailProof)
        );

        blueprintId = 1;
        proof = Proof({
            a: [uint256(1), uint256(2)],
            b: [[uint256(3), uint256(4)], [uint256(5), uint256(6)]],
            c: [uint256(7), uint256(8)]
        });
        publicOutputs[0] = uint256(uint160(alice));
        decodedPublicOutputs = '"to": blueprintId, "username": "John Smith"';
        proverEthAddressIdx = 0;

        domainName = "gmail.com";
        publicKeyHash = 0x0ea9c777dc7110e5a9e89b13f0cfc540e3845ba120b2b6dc24024d61488d4788;

        vm.startPrank(owner);
        zkEmailProof.addVerifier(address(verifier));
        dkimRegistry.setDKIMPublicKeyHash(domainName, publicKeyHash);
        vm.stopPrank();
    }
}
