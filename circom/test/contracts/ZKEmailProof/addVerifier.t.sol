// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {MockVerifier} from "../../../contracts/test/MockVerifier.sol";
import {BaseTest} from "./BaseTest.t.sol";

contract ZKEmailProof_AddVerifier_Test is BaseTest {
    function setUp() public override {
        super.setUp();
    }

    function test_ZKEmailProof_AddVerifier_AddsVerifier() public {
        address newVerifier = address(new MockVerifier());
        bool isVerifier = zkEmailProof.verifiers(newVerifier);
        assertFalse(isVerifier);

        vm.startPrank(owner);
        zkEmailProof.addVerifier(newVerifier);
        vm.stopPrank();

        isVerifier = zkEmailProof.verifiers(newVerifier);
        assertTrue(isVerifier);
    }
}
