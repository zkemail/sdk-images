// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

/**
 * @title Groth16 Verifier Interface (testBlueprint fixture)
 * @notice Same shape as {IGroth16Verifier}, sized to blueprint f63c7198-76b1-413c-b785-7655ebdaaec1's
 * 7-element public-signal layout. Kept as a separate, distinctly-named file so it is never confused
 * with a per-blueprint generated interface (see
 * ../../../../kusama-grant/milestone-2/03_verifier_interface_and_wrappers.md).
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
