// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {ZKEmailProof, Proof} from "../ZKEmailProof.sol";

/**
 * @title TestVerifier
 * @notice A verifier for testing purposes
 */
contract TestVerifier {
    address public soulboundNFT;

    constructor(address _soulboundNFT) {
        soulboundNFT = _soulboundNFT;
    }

    function verifyAndMint(
        uint256[2] memory a,
        uint256[2][2] memory b,
        uint256[2] memory c,
        uint256[] memory publicOutputs
    ) external returns (bool) {
        address to = address(uint160(publicOutputs[0]));
        uint256 blueprintId = 1;
        Proof memory proof = Proof(a, b, c);
        string memory decodedPublicOutputs = _decodePublicOutputs(
            publicOutputs
        );

        ZKEmailProof(soulboundNFT).safeMint(
            to,
            blueprintId,
            proof,
            publicOutputs,
            decodedPublicOutputs
        );
        return true;
    }

    function _decodePublicOutputs(
        uint256[] memory publicOutputs
    ) internal pure returns (string memory) {
        publicOutputs;
        return '"to": 1, "username": "John Smith"';
    }
}
