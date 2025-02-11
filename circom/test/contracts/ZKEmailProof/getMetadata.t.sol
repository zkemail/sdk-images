// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {ZKEmailProofMetadata} from "../../../contracts/ZKEmailProof.sol";
import {BaseTest} from "./BaseTest.t.sol";

contract ZKEmailProof_GetMetadata_Test is BaseTest {
    function setUp() public override {
        super.setUp();
    }

    function test_ZKEmailProof_getMetadata() public view {
        ZKEmailProofMetadata memory metadata = zkEmailProof.getMetadata(alice);
        assertEq(metadata.blueprintId, 0);
        assertEq(metadata.verifier, address(0));
        assertEq(metadata.proof.a[0], uint256(0));
        assertEq(metadata.proof.a[1], uint256(0));
        assertEq(metadata.proof.b[0][0], uint256(0));
        assertEq(metadata.proof.b[0][1], uint256(0));
        assertEq(metadata.proof.b[1][0], uint256(0));
        assertEq(metadata.proof.b[1][1], uint256(0));
        assertEq(metadata.proof.c[0], uint256(0));
        assertEq(metadata.proof.c[1], uint256(0));
        assertEq(metadata.publicOutputs, new uint256[](0));
        assertEq(metadata.decodedPublicOutputs, "");
    }
}
