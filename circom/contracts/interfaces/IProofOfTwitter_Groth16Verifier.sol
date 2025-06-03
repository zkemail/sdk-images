// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

interface IProofOfTwitter_Groth16Verifier {
    function verifyProof(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[8] calldata signals
    ) external view returns (bool);
}
