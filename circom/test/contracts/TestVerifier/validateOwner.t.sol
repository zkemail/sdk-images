// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {TestVerifier} from "../../../contracts/test/TestVerifier.sol";
import {BaseTest} from "../BaseTest.t.sol";

contract TestVerifier_ValidateOwner_Test is BaseTest {
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

    function test_TestVerifier_ValidateOwner_RevertWhen_OwnerNotInProof()
        public
    {
        // Change the address in the proof to mismatch 'to'
        publicOutputsFixedSize[
            toAddressStartIndex
        ] = 116992936385065960565912052140412177013034054326367951776368268607948945456;
        publicOutputsFixedSize[
            toAddressStartIndex + 1
        ] = 81467355455428621312079923;

        vm.expectRevert(TestVerifier.OwnerNotInProof.selector);
        verifier.exposed_validateOwner(
            publicOutputsFixedSize,
            toAddressStartIndex,
            to
        );
    }

    function test_TestVerifier_ValidateOwner_RevertWhen_ZeroAddress() public {
        publicOutputsFixedSize[toAddressStartIndex] = uint256(
            uint160(address(0))
        );

        vm.expectRevert(Strings.StringsInvalidAddressFormat.selector);
        verifier.exposed_validateOwner(
            publicOutputsFixedSize,
            toAddressStartIndex,
            to
        );
    }

    function test_TestVerifier_ValidateOwner_Success() public view {
        verifier.exposed_validateOwner(
            publicOutputsFixedSize,
            toAddressStartIndex,
            to
        );
    }
}
