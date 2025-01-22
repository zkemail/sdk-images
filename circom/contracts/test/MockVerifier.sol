// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {IGroth16Verifier} from "../IGroth16Verifier.sol";

contract MockVerifier is IGroth16Verifier {
    function verifyProof(
        uint[2] calldata /*_pA*/,
        uint[2][2] calldata /*_pB*/,
        uint[2] calldata /*_pC*/,
        uint[] calldata /*_pubSignals*/
    ) external pure returns (bool) {
        return true;
    }
}
