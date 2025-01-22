// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {BaseTest} from "./BaseTest.t.sol";

contract ZKEmailProof_RemoveVerifier_Test is BaseTest {
    function setUp() public override {
        super.setUp();
    }

    function test_ZKEmailProof_RemoveVerifier_RemovesVerifier() public {
        bool isVerifier = zkEmailProof.approvedVerifiers(address(verifier));
        assertTrue(isVerifier);

        vm.startPrank(owner);
        zkEmailProof.removeVerifier(address(verifier));
        vm.stopPrank();

        isVerifier = zkEmailProof.approvedVerifiers(address(verifier));
        assertFalse(isVerifier);
    }
}
