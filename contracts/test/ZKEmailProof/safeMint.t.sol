// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {ZKEmailProof, Proof, ZKEmailProofMetadata} from "../../src/ZKEmailProof.sol";

contract ZKEmailProof_SafeMint_Test is Test {
    ZKEmailProof public zkEmailProof;
    address public alice;
    address public bob;

    uint256 blueprintId;
    Proof proof;
    uint256[] publicOutputs = new uint256[](1);
    string decodedPublicOutputs;
    address verifier;

    function setUp() public {
        zkEmailProof = new ZKEmailProof();
        alice = address(1);
        bob = address(2);

        blueprintId = 1;
        proof = Proof({
            a: [uint256(1), uint256(2)],
            b: [[uint256(3), uint256(4)], [uint256(5), uint256(6)]],
            c: [uint256(7), uint256(8)]
        });
        publicOutputs[0] = uint256(uint160(alice));
        decodedPublicOutputs = '"to": 1, "username": "John Smith"';
        verifier = address(3);
    }

    function test_ZKEmailProof_SafeMint() public {
        vm.prank(alice);
        zkEmailProof.safeMint(
            alice,
            blueprintId,
            proof,
            publicOutputs,
            decodedPublicOutputs,
            verifier
        );

        uint256 tokenId = 0;
        assertEq(zkEmailProof.ownerOf(tokenId), alice);

        ZKEmailProofMetadata memory metadata = zkEmailProof.getMetadata(alice);
        assertEq(metadata.blueprintId, blueprintId);
        assertEq(metadata.proof.a[0], proof.a[0]);
        assertEq(metadata.proof.a[1], proof.a[1]);
        assertEq(metadata.proof.b[0][0], proof.b[0][0]);
        assertEq(metadata.proof.b[0][1], proof.b[0][1]);
        assertEq(metadata.proof.b[1][0], proof.b[1][0]);
        assertEq(metadata.proof.b[1][1], proof.b[1][1]);
        assertEq(metadata.proof.c[0], proof.c[0]);
        assertEq(metadata.proof.c[1], proof.c[1]);
        assertEq(metadata.publicOutputs, publicOutputs);
        assertEq(metadata.decodedPublicOutputs, decodedPublicOutputs);
        assertEq(metadata.verifier, verifier);
    }

    function test_ZKEmailProof_SafeMint_RevertWhen_OwnerNotInPublicOutputs()
        public
    {
        publicOutputs[0] = uint256(uint160(bob)); // invalid owner

        vm.prank(alice);
        vm.expectRevert(ZKEmailProof.OwnerNotInProof.selector);
        zkEmailProof.safeMint(
            alice,
            blueprintId,
            proof,
            publicOutputs,
            decodedPublicOutputs,
            verifier
        );
    }
}
