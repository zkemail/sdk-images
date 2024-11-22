// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {ZKEmailProof, ZKEmailProofMetadata} from "../../src/ZKEmailProof.sol";

contract ZKEmailProof_SafeTransferFrom_Test is Test {
    ZKEmailProof public zkEmailProof;
    address public alice;
    address public bob;

    function setUp() public {
        zkEmailProof = new ZKEmailProof();
        alice = address(1);
        bob = address(2);
    }

    function test_ZKEmailProof_SafeTransferFrom_RevertWhen_TransferToBob()
        public
    {
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

        vm.expectRevert(ZKEmailProof.CannotTransferSoulboundToken.selector);
        zkEmailProof.safeTransferFrom(alice, bob, tokenId);
    }
}
