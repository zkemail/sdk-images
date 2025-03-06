// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {IDKIMRegistry} from "@zk-email/contracts/interfaces/IDKIMRegistry.sol";
import {IGroth16Verifier} from "../../../contracts/interfaces/IGroth16Verifier.sol";
import {BaseTest} from "../BaseTest.t.sol";
import {ZKEmailProofMetadata} from "../../../contracts/ZKEmailProof.sol";
import {TestVerifier} from "../../../contracts/test/TestVerifier.sol";

contract TestVerifier_VerifyAndMint_Test is BaseTest {
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

    function test_TestVerifier_VerifyAndMint_RevertWhen_InvalidDKIMPublicKeyHash()
        public
    {
        publicOutputsFixedSize[0] = uint256(keccak256("invalid"));

        vm.expectRevert(TestVerifier.InvalidDKIMPublicKeyHash.selector);
        verifier.verifyAndMint(
            proof.a,
            proof.b,
            proof.c,
            publicOutputsFixedSize,
            publicOutputFieldNames,
            to,
            blueprintId,
            toAddressStartIndex
        );
    }

    function test_TestVerifier_VerifyAndMint_RevertWhen_InvalidProof() public {
        vm.mockCall(
            verifier.verifier(),
            abi.encodeWithSelector(IGroth16Verifier.verifyProof.selector),
            abi.encode(false)
        );

        vm.expectRevert(TestVerifier.InvalidProof.selector);
        verifier.verifyAndMint(
            proof.a,
            proof.b,
            proof.c,
            publicOutputsFixedSize,
            publicOutputFieldNames,
            to,
            blueprintId,
            toAddressStartIndex
        );
    }

    function test_TestVerifier_VerifyAndMint_Success() public {
        verifier.verifyAndMint(
            proof.a,
            proof.b,
            proof.c,
            publicOutputsFixedSize,
            publicOutputFieldNames,
            to,
            blueprintId,
            toAddressStartIndex
        );

        uint256 tokenId = 0;
        assertEq(zkEmailProof.balanceOf(alice), 1);
        assertEq(zkEmailProof.ownerOf(tokenId), alice);

        ZKEmailProofMetadata memory metadata = zkEmailProof.getMetadata(alice);
        assertEq(metadata.blueprintId, blueprintId);
        assertEq(metadata.verifier, address(verifier));
        assertEq(metadata.proof.a[0], proof.a[0]);
        assertEq(metadata.proof.a[1], proof.a[1]);
        assertEq(metadata.proof.b[0][0], proof.b[0][0]);
        assertEq(metadata.proof.b[0][1], proof.b[0][1]);
        assertEq(metadata.proof.b[1][0], proof.b[1][0]);
        assertEq(metadata.proof.b[1][1], proof.b[1][1]);
        assertEq(metadata.proof.c[0], proof.c[0]);
        assertEq(metadata.proof.c[1], proof.c[1]);
        assertEq(metadata.publicOutputs[0], publicOutputs[0]);
        assertEq(metadata.publicOutputs[1], publicOutputs[1]);
        assertEq(metadata.publicOutputs[2], publicOutputs[2]);
        assertEq(metadata.publicOutputs[3], publicOutputs[3]);
        assertEq(metadata.publicOutputs[4], publicOutputs[4]);
        assertEq(metadata.publicOutputs[5], publicOutputs[5]);
        assertEq(metadata.publicOutputs[6], publicOutputs[6]);
        assertEq(metadata.publicOutputs[7], publicOutputs[7]);
        assertEq(metadata.publicOutputs[8], publicOutputs[8]);
        assertEq(metadata.publicOutputs[9], publicOutputs[9]);
        assertEq(metadata.publicOutputs[10], publicOutputs[10]);
        assertEq(metadata.publicOutputs[11], publicOutputs[11]);
        assertEq(metadata.publicOutputs[12], publicOutputs[12]);
        assertEq(metadata.publicOutputs[13], publicOutputs[13]);
        assertEq(metadata.publicOutputs[14], publicOutputs[14]);
        assertEq(metadata.publicOutputs[15], publicOutputs[15]);
        assertEq(metadata.decodedPublicOutputs, decodedPublicOutputs);
    }
}
