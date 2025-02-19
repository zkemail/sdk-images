// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {IDKIMRegistry} from "@zk-email/contracts/interfaces/IDKIMRegistry.sol";
import {StringUtils} from "@zk-email/contracts/utils/StringUtils.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {IGroth16Verifier} from "../interfaces/IGroth16Verifier.sol";
import {ZKEmailProof, Proof} from "../ZKEmailProof.sol";

/**
 * @title TestVerifier based on ExtractGoogleDomain_Verifier
 */
contract TestVerifier {
    address public immutable dkimRegistry;
    address public immutable verifier;
    address public immutable zkEmailProof;

    uint256 public constant packSize = 31;
    string public constant domain = "accounts.google.com";
    uint16 public constant sender_domain_len = 3;
    uint16 public constant address_len = 1;

    error InvalidDKIMPublicKeyHash();
    error InvalidProof();
    error OwnerNotInProof();

    constructor(
        address _dkimRegistry,
        address _verifier,
        address _zkEmailProof
    ) {
        dkimRegistry = _dkimRegistry;
        verifier = _verifier;
        zkEmailProof = _zkEmailProof;
    }

    function verify(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[5] calldata publicOutputs
    ) external view {
        bytes32 publicKeyHash = bytes32(publicOutputs[0]);
        if (
            !IDKIMRegistry(dkimRegistry).isDKIMPublicKeyHashValid(
                domain,
                publicKeyHash
            )
        ) {
            revert InvalidDKIMPublicKeyHash();
        }
        IGroth16Verifier(verifier).verify(a, b, c, publicOutputs);
    }

    function verifyAndMint(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[5] calldata publicOutputs,
        string[2] calldata publicOutputFieldNames,
        address to,
        uint256 blueprintId,
        uint256 toAddressStartIndex
    ) external {
        bytes32 publicKeyHash = bytes32(publicOutputs[0]);
        if (
            !IDKIMRegistry(dkimRegistry).isDKIMPublicKeyHashValid(
                domain,
                publicKeyHash
            )
        ) {
            revert InvalidDKIMPublicKeyHash();
        }
        IGroth16Verifier(verifier).verify(a, b, c, publicOutputs);

        validateOwner(publicOutputs, toAddressStartIndex, to);

        Proof memory proof = Proof(a, b, c);

        uint256 publicOutputsLength = publicOutputs.length;
        uint256[] memory dynamicSignals = new uint256[](publicOutputsLength);
        for (uint256 i = 0; i < publicOutputsLength; i++) {
            dynamicSignals[i] = publicOutputs[i];
        }

        string memory decodedPublicOutputs = decodePublicOutputs(
            publicOutputFieldNames,
            publicOutputs
        );

        ZKEmailProof(zkEmailProof).mintProof(
            to,
            blueprintId,
            proof,
            dynamicSignals,
            decodedPublicOutputs
        );
    }

    function decodePublicOutputs(
        string[2] calldata publicOutputFieldNames,
        uint256[5] calldata publicOutputs
    ) internal pure returns (string memory) {
        uint256[] memory packed_sender_domain = new uint256[](
            sender_domain_len
        );
        for (uint256 i = 0; i < sender_domain_len; i++) {
            packed_sender_domain[i] = publicOutputs[1 + i];
        }
        string memory sender_domain_string = StringUtils
            .convertPackedBytesToString(
                packed_sender_domain,
                packSize * sender_domain_len,
                packSize
            );

        uint256 fieldsLength = publicOutputFieldNames.length;
        string[] memory jsonFields = new string[](fieldsLength);
        for (uint256 i = 0; i < fieldsLength; i++) {
            jsonFields[i] = string.concat(
                '"',
                publicOutputFieldNames[i],
                '":"',
                sender_domain_string,
                '"'
            );
        }
        string memory jsonFieldsString;
        for (uint256 i = 0; i < fieldsLength; i++) {
            if (i < fieldsLength - 1) {
                jsonFieldsString = string.concat(
                    jsonFieldsString,
                    jsonFields[i],
                    ","
                );
            }
            jsonFieldsString = string.concat(jsonFieldsString, jsonFields[i]);
        }

        string memory flattenedJson = string.concat("{", jsonFieldsString, "}");
        return flattenedJson;
    }

    function validateOwner(
        uint256[5] memory publicOutputs,
        uint256 toAddressStartIndex,
        address to
    ) internal pure {
        uint256[] memory packed_address = new uint256[](address_len);
        for (uint256 i = 0; i < address_len; i++) {
            packed_address[i] = publicOutputs[toAddressStartIndex + i];
        }
        string memory toAddressString = StringUtils.convertPackedBytesToString(
            packed_address,
            packSize * address_len,
            packSize
        );

        // Owner should be committed to in each proof. This prevents
        // frontrunning `mintProof` with a valid proof but malicious "to" address.
        // An entity could also just mint the proof many times for different accounts
        // if (address(uint160(publicOutputs[toAddressStartIndex])) != to) {
        // if (toAddressString.parseAddress() != to) {
        if (Strings.parseAddress(toAddressString) != to) {
            revert OwnerNotInProof();
        }
    }
}
