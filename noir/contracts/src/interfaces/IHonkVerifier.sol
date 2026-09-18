// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

/**
 * @title Honk Verifier Interface
 * @notice Interface for verifying Honk proofs
 */
interface IHonkVerifier {
    /**
     * @notice Verifies a Honk proof
     * @param proof The proof to verify
     * @param publicInputs The public inputs to verify
     * @return True if the proof is valid, false otherwise
     */
    function verify(bytes calldata proof, bytes32[] calldata publicInputs) external view returns (bool);
}
