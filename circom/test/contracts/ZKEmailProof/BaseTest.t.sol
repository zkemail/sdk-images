// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {DKIMRegistry} from "@zk-email/contracts/DKIMRegistry.sol";
import {ZKEmailProof, Proof} from "../../../contracts/ZKEmailProof.sol";
import {MockVerifier} from "../../../contracts/test/MockVerifier.sol";

contract BaseTest is Test {
    ZKEmailProof zkEmailProof;
    address dkimRegistry;
    address verifier;

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

        dkimRegistry = address(new DKIMRegistry(owner));
        verifier = address(new MockVerifier());
        zkEmailProof = new ZKEmailProof(owner, dkimRegistry);

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
        zkEmailProof.addVerifier(verifier);
        DKIMRegistry(dkimRegistry).setDKIMPublicKeyHash(
            domainName,
            publicKeyHash
        );
        vm.stopPrank();
    }
}
