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

/**
 * @title ZKEmailProof
 * @notice A soulbound NFT contract that represents valid ZK Email proofs
 */
contract ZKEmailProof is ERC721, ERC721URIStorage {
    using Strings for uint256;
    using Strings for address;

    error OwnerNotInProof();
    error CannotTransferSoulboundToken();

    uint256 private _nextTokenId;

    mapping(address owner => ZKEmailProofMetadata metadata)
        private ownerToMetadata;

    constructor() ERC721("ZKEmailProof", "ZKEP") {}

    /**
     * @notice Mints a new soulbound NFT representing a ZK email proof
     * @dev First element of proof must be the recipient address
     * @param to Address to mint the NFT to
     * @param blueprintId ID of the blueprint used for the proof
     * @param proof Proof
     * @param verifier Address of the verifier contract
     * @param publicOutputs String of public outputs from the proof
     */
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

    /**
     * @notice Generates the token URI containing metadata for a given token
     * @param tokenId ID of the token to generate the URI for
     * @return Base64 encoded JSON metadata string
     */
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

    /**
     * @notice Checks if contract supports an interface
     * @param interfaceId Interface identifier to check
     * @return bool indicating if interface is supported
     */
    function supportsInterface(
        bytes4 interfaceId
    ) public view override(ERC721, ERC721URIStorage) returns (bool) {
        return super.supportsInterface(interfaceId);
    }

    /**
     * @notice Gets the metadata for a given owner's NFT
     * @param owner Address to get metadata for
     * @return ZKEmailProofMetadata struct containing the NFT metadata
     */
    function getMetadata(
        address owner
    ) public view returns (ZKEmailProofMetadata memory) {
        return ownerToMetadata[owner];
    }

    /**
     * @notice Internal function to handle token transfers
     * @dev Overridden to prevent token transfers, thus making the NFTs soulbound
     * @param to Address to transfer to
     * @param tokenId ID of token being transferred
     * @param auth Address authorized to make transfer
     * @return Address token was transferred from
     */
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
