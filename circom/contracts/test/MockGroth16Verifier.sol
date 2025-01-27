// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {IVerifier} from "../interfaces/IVerifier.sol";

contract MockGroth16Verifier is IVerifier {
    function verify(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[5] calldata signals
    ) external pure {
        a;
        b;
        c;
        signals;
    }
}
