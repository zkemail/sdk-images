// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

/**
 * @title Groth16 Verifier Interface (testBlueprint fixture)
 * @notice Byte-for-byte copy of the committed test fixture at
 * test/fixtures/testBlueprint/ITestBlueprintGroth16Verifier.sol, sized to blueprint
 * f63c7198-76b1-413c-b785-7655ebdaaec1's 7-element public-signal layout. Deployed here to Paseo as
 * part of the milestone-2 E2E demo -- see ../../../kusama-grant/milestone-2/06_e2e_demo.md.
 */
interface ITestBlueprintGroth16Verifier {
    function verifyProof(
        uint256[2] calldata pA,
        uint256[2][2] calldata pB,
        uint256[2] calldata pC,
        uint256[7] calldata publicSignals
    )
        external
        view
        returns (bool);
}
