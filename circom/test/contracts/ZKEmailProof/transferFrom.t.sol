// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {ZKEmailProof} from "../../../contracts/ZKEmailProof.sol";

import {BaseTest} from "./BaseTest.t.sol";

contract ZKEmailProof_SafeTransferFrom_Test is BaseTest {
    function setUp() public override {
        super.setUp();
    }

    function test_ZKEmailProof_SafeTransferFrom_RevertWhen_TransferToBob()
        public
    {
        vm.prank(address(verifier));
        zkEmailProof.mintProof(
            alice,
            blueprintId,
            proof,
            publicOutputs,
            decodedPublicOutputs
        );
        uint256 tokenId = 0;

        vm.expectRevert(ZKEmailProof.CannotTransferSoulboundToken.selector);
        zkEmailProof.safeTransferFrom(alice, bob, tokenId);
    }

    function test_ZKEmailProof_TransferFrom_RevertWhen_TransferToBob() public {
        vm.prank(address(verifier));
        zkEmailProof.mintProof(
            alice,
            blueprintId,
            proof,
            publicOutputs,
            decodedPublicOutputs
        );
        uint256 tokenId = 0;

        vm.expectRevert(ZKEmailProof.CannotTransferSoulboundToken.selector);
        zkEmailProof.transferFrom(alice, bob, tokenId);
    }
}
