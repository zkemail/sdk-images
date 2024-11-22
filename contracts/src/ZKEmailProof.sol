// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {ERC721URIStorage} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

struct ZKEmailProofMetadata {
    uint256 blueprintId;
    uint256[] proof;
    address verifier;
    string publicOutputs;
}

contract ZKEmailProof is ERC721, ERC721URIStorage {
    using Strings for uint256;
    using Strings for address;

    error OwnerNotInProof();
    error CannotTransferSoulboundToken();

    uint256 private _nextTokenId;

    mapping(address owner => ZKEmailProofMetadata metadata)
        private ownerToMetadata;

    constructor() ERC721("ZKEmailProof", "ZKEP") {}

    function safeMint(
        address to,
        uint256 blueprintId,
        uint256[] memory proof,
        address verifier,
        string memory publicOutputs
    ) public {
        // Owner should be committed to in each proof. This prevents
        // frontrunning safeMint with a valid proof but malicious "to" address
        if (address(uint160(proof[0])) != to) {
            revert OwnerNotInProof();
        }

        ownerToMetadata[to] = ZKEmailProofMetadata({
            blueprintId: blueprintId,
            proof: proof,
            verifier: verifier,
            publicOutputs: publicOutputs
        });

        uint256 tokenId = _nextTokenId++;
        _safeMint(to, tokenId);
        _setTokenURI(tokenId, tokenURI(tokenId));
    }

    function tokenURI(
        uint256 tokenId
    ) public view override(ERC721, ERC721URIStorage) returns (string memory) {
        address owner = ownerOf(tokenId);
        ZKEmailProofMetadata memory metadata = ownerToMetadata[owner];

        string memory proofJson = "[";
        for (uint256 i = 0; i < metadata.proof.length; i++) {
            proofJson = string.concat(proofJson, metadata.proof[i].toString());
            if (i < metadata.proof.length - 1) {
                proofJson = string.concat(proofJson, ",");
            }
        }
        proofJson = string.concat(proofJson, "]");

        string memory dataURI = string.concat(
            "{",
            '"name": "ZKEmailProof NFT #',
            tokenId.toString(),
            '",',
            '"description": "Soulbound NFT representing a valid ZK Email proof for an account",',
            '"attributes": [',
            '{ "trait_type": "Blueprint ID", "value": "',
            metadata.blueprintId.toString(),
            '" },',
            '{ "trait_type": "Verifier", "value": "',
            metadata.verifier.toHexString(),
            '" },',
            '{ "trait_type": "Public Outputs", "value": "',
            metadata.publicOutputs,
            '" },',
            '{ "trait_type": "Proof", "value": ',
            proofJson,
            " }",
            "]",
            "}"
        );

        return
            string.concat(
                "data:application/json;base64,",
                Base64.encode(bytes(dataURI))
            );
    }

    function supportsInterface(
        bytes4 interfaceId
    ) public view override(ERC721, ERC721URIStorage) returns (bool) {
        return super.supportsInterface(interfaceId);
    }

    function getMetadata(
        address owner
    ) public view returns (ZKEmailProofMetadata memory) {
        return ownerToMetadata[owner];
    }

    function _update(
        address to,
        uint256 tokenId,
        address auth
    ) internal override returns (address) {
        address from = _ownerOf(tokenId);
        if (from != address(0)) {
            revert CannotTransferSoulboundToken();
        }
        return super._update(to, tokenId, auth);
    }
}
