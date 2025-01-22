// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IVerifier {
    function verify(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[5] calldata signals
    ) external view;
}
