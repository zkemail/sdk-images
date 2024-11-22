// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {ZKEmailProof, ZKEmailProofMetadata} from "../../src/ZKEmailProof.sol";

contract ZKEmailProof_SafeMint_Test is Test {
    ZKEmailProof public zkEmailProof;
    address public alice;
    address public bob;

    function setUp() public {
        zkEmailProof = new ZKEmailProof();
        alice = address(1);
        bob = address(2);
    }

    function test_ZKEmailProof_SafeMint() public {
        uint256 blueprintId = 1;
        uint256[] memory proof = new uint256[](1);
        proof[0] = uint256(uint160(alice));
        address verifier = address(3);
        string memory publicOutputs = "Test Public Outputs";

        vm.prank(alice);
        zkEmailProof.safeMint(
            alice,
            blueprintId,
            proof,
            verifier,
            publicOutputs
        );

        uint256 tokenId = 0;
        assertEq(zkEmailProof.ownerOf(tokenId), alice);

        ZKEmailProofMetadata memory metadata = zkEmailProof.getMetadata(alice);
        assertEq(metadata.blueprintId, blueprintId);
        assertEq(metadata.proof[0], proof[0]);
        assertEq(metadata.verifier, verifier);
        assertEq(metadata.publicOutputs, publicOutputs);
    }

    function test_ZKEmailProof_SafeMint_RevertWhen_OwnerNotInProof() public {
        uint256 blueprintId = 1;
        uint256[] memory proof = new uint256[](1);
        proof[0] = uint256(uint160(bob)); // invalid owner
        address verifier = address(0x789);
        string memory publicOutputs = "Test Public Outputs";

        vm.prank(alice);
        vm.expectRevert(ZKEmailProof.OwnerNotInProof.selector);
        zkEmailProof.safeMint(
            alice,
            blueprintId,
            proof,
            verifier,
            publicOutputs
        );
    }
}
