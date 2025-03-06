// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {TestVerifier} from "../../contracts/test/TestVerifier.sol";

/**
 * @title TestVerifierHarness based on TestVerifier
 */
contract TestVerifierHarness is TestVerifier {
    constructor(
        address _dkimRegistry,
        address _verifier,
        address _zkEmailProof
    ) TestVerifier(_dkimRegistry, _verifier, _zkEmailProof) {}

    function exposed_decodePublicOutputs(
        string[3] calldata publicOutputFieldNames,
        uint256[16] calldata publicOutputs
    ) public pure returns (string memory) {
        return decodePublicOutputs(publicOutputFieldNames, publicOutputs);
    }
    function exposed_convertPackedFieldsToPublicOutput(
        uint256[16] memory publicOutputs,
        uint256 startIndex,
        string memory publicOutputFieldName,
        uint256 field_len
    ) public pure returns (string memory) {
        return
            convertPackedFieldsToPublicOutput(
                publicOutputs,
                startIndex,
                publicOutputFieldName,
                field_len
            );
    }

    function exposed_validateOwner(
        uint256[16] memory publicOutputs,
        uint256 toAddressStartIndex,
        address to
    ) public pure {
        validateOwner(publicOutputs, toAddressStartIndex, to);
    }
}
