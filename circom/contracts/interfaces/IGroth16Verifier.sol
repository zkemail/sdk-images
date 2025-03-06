// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

interface IGroth16Verifier {
    function verifyProof(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[16] calldata signals
    ) external view returns (bool);
}
