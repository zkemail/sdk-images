// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

import {TestVerifier} from "../../../contracts/test/TestVerifier.sol";
import {BaseTest} from "../BaseTest.t.sol";

contract ZKEmailProof_AddVerifier_Test is BaseTest {
    function setUp() public override {
        super.setUp();
    }

    function test_ZKEmailProof_AddVerifier_RevertWhen_NotOwner() public {
        address newVerifier = address(
            new TestVerifier(
                address(dkimRegistry),
                address(groth16Verifier),
                address(zkEmailProof)
            )
        );

        vm.prank(alice);
        vm.expectRevert(
            abi.encodeWithSelector(
                Ownable.OwnableUnauthorizedAccount.selector,
                alice
            )
        );
        zkEmailProof.addVerifier(newVerifier);
    }

    function test_ZKEmailProof_AddVerifier_AddsVerifier() public {
        address newVerifier = address(
            new TestVerifier(
                address(dkimRegistry),
                address(groth16Verifier),
                address(zkEmailProof)
            )
        );
        bool isVerifier = zkEmailProof.approvedVerifiers(newVerifier);
        assertFalse(isVerifier);

        vm.startPrank(owner);
        zkEmailProof.addVerifier(newVerifier);
        vm.stopPrank();

        isVerifier = zkEmailProof.approvedVerifiers(newVerifier);
        assertTrue(isVerifier);
    }
}
