// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {IDKIMRegistry} from "@zk-email/contracts/interfaces/IDKIMRegistry.sol";
import {IGroth16Verifier} from "../../../contracts/interfaces/IGroth16Verifier.sol";
import {BaseTest} from "../BaseTest.t.sol";
import {TestVerifier} from "../../../contracts/test/TestVerifier.sol";

contract TestVerifier_Verify_Test is BaseTest {
    // verifier expects a fixed size array
    uint256[16] publicOutputsFixedSize;

    function setUp() public override {
        super.setUp();
        publicOutputsFixedSize = [
            publicOutputs[0],
            publicOutputs[1],
            publicOutputs[2],
            publicOutputs[3],
            publicOutputs[4],
            publicOutputs[5],
            publicOutputs[6],
            publicOutputs[7],
            publicOutputs[8],
            publicOutputs[9],
            publicOutputs[10],
            publicOutputs[11],
            publicOutputs[12],
            publicOutputs[13],
            publicOutputs[14],
            publicOutputs[15]
        ];
    }

    function test_TestVerifier_Verify_RevertWhen_InvalidDKIMPublicKeyHash()
        public
    {
        publicOutputsFixedSize[0] = uint256(keccak256("invalid"));

        vm.expectRevert(TestVerifier.InvalidDKIMPublicKeyHash.selector);
        verifier.verify(proof.a, proof.b, proof.c, publicOutputsFixedSize);
    }

    function test_TestVerifier_Verify_RevertWhen_InvalidProof() public {
        vm.mockCall(
            verifier.verifier(),
            abi.encodeWithSelector(IGroth16Verifier.verifyProof.selector),
            abi.encode(false)
        );

        vm.expectRevert(TestVerifier.InvalidProof.selector);
        verifier.verify(proof.a, proof.b, proof.c, publicOutputsFixedSize);
    }

    function test_TestVerifier_Verify_Success() public view {
        verifier.verify(proof.a, proof.b, proof.c, publicOutputsFixedSize);
    }
}
