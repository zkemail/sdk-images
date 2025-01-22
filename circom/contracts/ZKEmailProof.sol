// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {console} from "forge-std/console.sol";
import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {ERC721URIStorage} from "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";

struct Proof {
    uint256[2] a;
    uint256[2][2] b;
    uint256[2] c;
}

struct ZKEmailProofMetadata {
    uint256 blueprintId;
    address verifier;
    Proof proof;
    uint256[] publicOutputs;
    string decodedPublicOutputs;
}

/**
 * @title ZKEmailProof
 * @notice A soulbound NFT contract that represents valid ZK Email proofs
 */
contract ZKEmailProof is ERC721, Ownable {
    using Strings for uint256;
    using Strings for address;

    error OwnerNotInProof();
    error CannotTransferSoulboundToken();
    error OnlyVerifier();
    error InvalidVerifier();

    uint256 private _nextTokenId;

    mapping(address verifier => bool approved) public approvedVerifiers;
    mapping(address => ZKEmailProofMetadata) private _ownerToMetadata;

    modifier onlyVerifier() {
        if (!approvedVerifiers[msg.sender]) {
            revert OnlyVerifier();
        }
        _;
    }

    constructor(
        address _initialOwner
    ) ERC721("ZKEmailProof", "ZKEP") Ownable(_initialOwner) {}

    /**
     * @notice Adds a verifier contract. Can only be called by the owner
     * @param _verifier The new verifier contract address
     */
    function addVerifier(address _verifier) external onlyOwner {
        if (_verifier == address(0)) {
            revert InvalidVerifier();
        }
        approvedVerifiers[_verifier] = true;
    }

    /**
     * @notice Removes a verifier contract. Can only be called by the owner
     * @param _verifier The verifier contract address to remove
     */
    function removeVerifier(address _verifier) external onlyOwner {
        if (!approvedVerifiers[_verifier]) {
            revert InvalidVerifier();
        }
        approvedVerifiers[_verifier] = false;
    }

    /**
     * @notice Mints a new soulbound NFT representing a ZK email proof
     * @dev First element of publicOutputs must be the recipient address
     * @param to Address to mint the NFT to
     * @param blueprintId ID of the blueprint used for the proof
     * @param proof Proof struct
     * @param publicOutputs uint256[] of public outputs
     * @param decodedPublicOutputs Decoded public outputs as flattened json
     * @param toAddressIndex Index of the to address in the publicOutputs array
     */
    function mintProof(
        address to,
        uint256 blueprintId,
        Proof memory proof,
        uint256[] memory publicOutputs,
        string memory decodedPublicOutputs,
        uint256 toAddressIndex
    ) public onlyVerifier {
        // Owner should be committed to in each proof. This prevents
        // frontrunning safeMint with a valid proof but malicious "to" address
        if (address(uint160(publicOutputs[toAddressIndex])) != to) {
            revert OwnerNotInProof();
        }

        _ownerToMetadata[to] = ZKEmailProofMetadata({
            blueprintId: blueprintId,
            verifier: msg.sender,
            proof: proof,
            publicOutputs: publicOutputs,
            decodedPublicOutputs: decodedPublicOutputs
        });

        uint256 tokenId = _nextTokenId++;
        _safeMint(to, tokenId);
    }

    // Override required by Solidity for multiple inheritance
    function supportsInterface(
        bytes4 interfaceId
    ) public view override(ERC721) returns (bool) {
        return super.supportsInterface(interfaceId);
    }

    function tokenURI(
        uint256 tokenId
    ) public view override returns (string memory) {
        address owner = ownerOf(tokenId);
        ZKEmailProofMetadata memory metadata = _ownerToMetadata[owner];

        string memory baseJson = string.concat(
            '{"name":"ZKEmail Proof #',
            tokenId.toString(),
            '","description":"Soulbound NFT representing a valid ZK Email proof for an account","attributes":['
        );

        string memory attributes = string.concat(
            '{"trait_type":"Blueprint ID","value":"',
            metadata.blueprintId.toString(),
            '"},',
            _buildProofJson(metadata.proof),
            ',{"trait_type":"Public Outputs","value":',
            _buildPublicOutputsJson(metadata.publicOutputs),
            "},",
            '{"trait_type":"Decoded Public Outputs","value":{',
            metadata.decodedPublicOutputs,
            "}},",
            '{"trait_type":"Verifier","value":"',
            metadata.verifier.toHexString(),
            '"}]}'
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
                '{"trait_type":"Proof_a","value":[',
                proof.a[0].toString(),
                ",",
                proof.a[1].toString(),
                "]},",
                '{"trait_type":"Proof_b","value":[[',
                proof.b[0][0].toString(),
                ",",
                proof.b[0][1].toString(),
                "],[",
                proof.b[1][0].toString(),
                ",",
                proof.b[1][1].toString(),
                "]]},",
                '{"trait_type":"Proof_c","value":[',
                proof.c[0].toString(),
                ",",
                proof.c[1].toString(),
                "]}"
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
        string memory result = '"[';
        for (uint256 i = 0; i < publicOutputs.length; i++) {
            result = string.concat(result, publicOutputs[i].toString());
            if (i < publicOutputs.length - 1) {
                result = string.concat(result, ",");
            }
        }
        return string.concat(result, ']"');
    }

    /**
     * @notice Gets the metadata for a given owner's NFT
     * @param owner Address to get metadata for
     * @return ZKEmailProofMetadata struct containing the NFT metadata
     */
    function getMetadata(
        address owner
    ) public view returns (ZKEmailProofMetadata memory) {
        return _ownerToMetadata[owner];
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
