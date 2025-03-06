// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {BaseTest} from "../BaseTest.t.sol";
import {TestVerifier} from "../../../contracts/test/TestVerifier.sol";

contract TestVerifier_DecodePublicOutputs_Test is BaseTest {
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

    function test_TestVerifier_DecodePublicOutputs_Success() public view {
        string
            memory expectedOutput = '{"recipient_name":"John","proposal_title":"<em>Making Smart Accounts easy with ZK Email</em>","rejection_line":"we were unable to accept this submission"}';

        string memory decodedOutput = verifier.exposed_decodePublicOutputs(
            publicOutputFieldNames,
            publicOutputsFixedSize
        );

        assertEq(decodedOutput, expectedOutput);
    }

    function test_TestVerifier_DecodePublicOutputs_EmptyFields() public {
        publicOutputsFixedSize[1] = 0;

        vm.expectRevert(
            "No packed bytes found! Invalid final state of packed bytes in email; value is likely 0!"
        );
        verifier.exposed_decodePublicOutputs(
            publicOutputFieldNames,
            publicOutputsFixedSize
        );
    }

    // console.log("decodedOutput", decodedOutput);
    function test_TestVerifier_DecodePublicOutputs_SpecialCharacters() public {
        string memory expectedOutput;
        string memory decodedOutput;

        // recipient name with special chars - bytes should be in reverse order
        publicOutputsFixedSize[1] = uint256(bytes32("%$#@!nhoJ"));
        expectedOutput = '{"recipient_name":"John!@#$","proposal_title":"<em>Making Smart Accounts easy with ZK Email</em>","rejection_line":"we were unable to accept this submission"}';
        decodedOutput = verifier.exposed_decodePublicOutputs(
            publicOutputFieldNames,
            publicOutputsFixedSize
        );
        assertEq(decodedOutput, expectedOutput);

        // proposal title with HTML - bytes should be in reverse order
        publicOutputsFixedSize[1] = uint256(bytes32(" >tpircs/<>tpircs<"));
        expectedOutput = '{"recipient_name":"<script></script>","proposal_title":"<em>Making Smart Accounts easy with ZK Email</em>","rejection_line":"we were unable to accept this submission"}';
        decodedOutput = verifier.exposed_decodePublicOutputs(
            publicOutputFieldNames,
            publicOutputsFixedSize
        );
        assertEq(decodedOutput, expectedOutput);

        // rejection line with escape chars - bytes should be in reverse order
        publicOutputsFixedSize[1] = uint256(bytes32(" 'ecnetnes depacsE'"));
        expectedOutput = '{"recipient_name":"\'Escaped sentence\'","proposal_title":"<em>Making Smart Accounts easy with ZK Email</em>","rejection_line":"we were unable to accept this submission"}';
        decodedOutput = verifier.exposed_decodePublicOutputs(
            publicOutputFieldNames,
            publicOutputsFixedSize
        );
        assertEq(decodedOutput, expectedOutput);
    }
}
