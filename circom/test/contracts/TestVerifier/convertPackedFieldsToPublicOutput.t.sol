// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {BaseTest} from "../BaseTest.t.sol";
import {TestVerifier} from "../../../contracts/test/TestVerifier.sol";

contract TestVerifier_ConvertPackedFieldsToPublicOutput_Test is BaseTest {
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

    function test_TestVerifier_ConvertPackedFieldsToPublicOutput_Success()
        public
        view
    {
        uint256 startIndex = 1;
        string memory publicOutputFieldName = publicOutputFieldNames[0];
        uint256 fieldLength = verifier.recipient_name_len();

        string memory expectedResult = '"recipient_name":"John"';
        string memory result = verifier
            .exposed_convertPackedFieldsToPublicOutput(
                publicOutputsFixedSize,
                startIndex,
                publicOutputFieldName,
                fieldLength
            );
        assertEq(result, expectedResult);

        startIndex = 1 + verifier.recipient_name_len();
        publicOutputFieldName = publicOutputFieldNames[1];
        fieldLength = verifier.proposal_title_len();

        expectedResult = '"proposal_title":"<em>Making Smart Accounts easy with ZK Email</em>"';
        result = verifier.exposed_convertPackedFieldsToPublicOutput(
            publicOutputsFixedSize,
            startIndex,
            publicOutputFieldName,
            fieldLength
        );
        assertEq(result, expectedResult);

        startIndex =
            1 +
            verifier.recipient_name_len() +
            verifier.proposal_title_len();
        publicOutputFieldName = publicOutputFieldNames[2];
        fieldLength = verifier.rejection_line_len();

        expectedResult = '"rejection_line":"we were unable to accept this submission"';
        result = verifier.exposed_convertPackedFieldsToPublicOutput(
            publicOutputsFixedSize,
            startIndex,
            publicOutputFieldName,
            fieldLength
        );
        assertEq(result, expectedResult);
    }

    function test_TestVerifier_ConvertPackedFieldsToPublicOutput_EmptyField()
        public
    {
        uint256 startIndex = 1 + verifier.recipient_name_len();
        string memory publicOutputFieldName = publicOutputFieldNames[1];
        uint256 fieldLength = verifier.proposal_title_len();

        for (uint i = startIndex; i < startIndex + fieldLength; i++) {
            publicOutputsFixedSize[i] = 0;
        }

        vm.expectRevert(
            "No packed bytes found! Invalid final state of packed bytes in email; value is likely 0!"
        );
        verifier.exposed_convertPackedFieldsToPublicOutput(
            publicOutputsFixedSize,
            startIndex,
            publicOutputFieldName,
            fieldLength
        );
    }
}
