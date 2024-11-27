// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {ERC721URIStorage} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

struct Proof {
    uint256[2] a;
    uint256[2][2] b;
    uint256[2] c;
}

struct ZKEmailProofMetadata {
    uint256 blueprintId;
    Proof proof;
    uint256[] publicOutputs;
    string decodedPublicOutputs;
    address verifier;
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
     * @param publicOutputs uint256[] of public outputs
     * @param decodedPublicOutputs Decoded public outputs as flattened json
     * @param verifier Address of the verifier contract
     */
    function safeMint(
        address to,
        uint256 blueprintId,
        Proof memory proof,
        uint256[] memory publicOutputs,
        string memory decodedPublicOutputs,
        address verifier
    ) public {
        // Owner should be committed to in each proof. This prevents
        // frontrunning safeMint with a valid proof but malicious "to" address
        if (address(uint160(publicOutputs[0])) != to) {
            revert OwnerNotInProof();
        }

        ownerToMetadata[to] = ZKEmailProofMetadata({
            blueprintId: blueprintId,
            proof: proof,
            publicOutputs: publicOutputs,
            decodedPublicOutputs: decodedPublicOutputs,
            verifier: verifier
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

        string memory baseJson = string.concat(
            '{"name": "ZKEmailProof NFT #',
            tokenId.toString(),
            '","description": "Soulbound NFT representing a valid ZK Email proof for an account","attributes": ['
        );

        string memory attributes = string.concat(
            '{ "trait_type": "Blueprint ID", "value": "',
            metadata.blueprintId.toString(),
            '" },',
            _buildProofJson(metadata.proof),
            '{ "trait_type": "Public Outputs", "value": ',
            _buildPublicOutputsJson(metadata.publicOutputs),
            " },",
            '{ "trait_type": "Decoded Public Outputs", "value": {',
            metadata.decodedPublicOutputs,
            "} },",
            '{ "trait_type": "Verifier", "value": "',
            metadata.verifier.toHexString(),
            '" }]}'
        );

        return
            string.concat(
                "data:application/json;base64,",
                Base64.encode(bytes(string.concat(baseJson, attributes)))
            );
    }

    /**
     * @notice Builds JSON string for proof
     * @param proof The Proof struct containing a, b, and c
     * @return JSON string representation of the proof
     */
    function _buildProofJson(
        Proof memory proof
    ) private pure returns (string memory) {
        return
            string.concat(
                '{ "trait_type": "Proof_a", "value": [',
                proof.a[0].toString(),
                ",",
                proof.a[1].toString(),
                "] },",
                '{ "trait_type": "Proof_b", "value": [[',
                proof.b[0][0].toString(),
                ",",
                proof.b[0][1].toString(),
                "],[",
                proof.b[1][0].toString(),
                ",",
                proof.b[1][1].toString(),
                "]] },",
                '{ "trait_type": "Proof_c", "value": [',
                proof.c[0].toString(),
                ",",
                proof.c[1].toString(),
                "] },"
            );
    }

    /**
     * @notice Converts public outputs array to JSON string
     * @param publicOutputs Array of public output values
     * @return JSON string representation of public outputs
     */
    function _buildPublicOutputsJson(
        uint256[] memory publicOutputs
    ) private pure returns (string memory) {
        string memory result = "[";
        for (uint256 i = 0; i < publicOutputs.length; i++) {
            result = string.concat(result, publicOutputs[i].toString());
            if (i < publicOutputs.length - 1) {
                result = string.concat(result, ",");
            }
        }
        return string.concat(result, "]");
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
