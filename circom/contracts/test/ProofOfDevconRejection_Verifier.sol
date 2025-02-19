// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/console.sol";
import {IDKIMRegistry} from "@zk-email/contracts/interfaces/IDKIMRegistry.sol";
import {StringUtils} from "@zk-email/contracts/utils/StringUtils.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {IProofOfDevconRejection_Groth16Verifier} from "../interfaces/IProofOfDevconRejection_Groth16Verifier.sol";
import {ZKEmailProof, Proof} from "../ZKEmailProof.sol";

/**
 * @title ProofOfDevconRejection_Verifier
 * @notice Test verifier that "Verifies that an applicant received a rejection email for their Devcon proposal."
 */
contract ProofOfDevconRejection_Verifier {
    address public immutable dkimRegistry;
    address public immutable verifier;
    address public immutable zkEmailProof;

    uint16 public constant packSize = 31;
    string public constant domain = "devcon.org";

    uint16 public constant recipient_name_len = 3;
    uint16 public constant proposal_title_len = 7;
    uint16 public constant rejection_line_len = 3;
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
        uint256[14] calldata publicOutputs
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
        if (
            !IProofOfDevconRejection_Groth16Verifier(verifier).verifyProof(
                a,
                b,
                c,
                publicOutputs
            )
        ) revert InvalidProof();
    }

    function verifyAndMint(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[14] calldata publicOutputs,
        string[3] calldata publicOutputFieldNames,
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
        if (
            !IProofOfDevconRejection_Groth16Verifier(verifier).verifyProof(
                a,
                b,
                c,
                publicOutputs
            )
        ) revert InvalidProof();

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
        string[3] calldata publicOutputFieldNames,
        uint256[14] calldata publicOutputs
    ) internal pure returns (string memory) {
        uint256 startIndex = 1;
        string memory recipient_name_string = convertPackedFieldsToPublicOutput(
            publicOutputs,
            startIndex,
            publicOutputFieldNames[0],
            recipient_name_len
        );

        startIndex += recipient_name_len;
        string memory proposal_title_string = convertPackedFieldsToPublicOutput(
            publicOutputs,
            startIndex,
            publicOutputFieldNames[1],
            proposal_title_len
        );

        startIndex += proposal_title_len;
        string memory rejection_line_string = convertPackedFieldsToPublicOutput(
            publicOutputs,
            startIndex,
            publicOutputFieldNames[2],
            rejection_line_len
        );

        return
            string.concat(
                "{",
                recipient_name_string,
                ",",
                proposal_title_string,
                ",",
                rejection_line_string,
                "}"
            );
    }

    function convertPackedFieldsToPublicOutput(
        uint256[14] memory publicOutputs,
        uint256 startIndex,
        string memory publicOutputFieldName,
        uint256 field_len
    ) internal pure returns (string memory) {
        uint256[] memory packed_field = new uint256[](field_len);
        for (uint256 i = 0; i < field_len; i++) {
            packed_field[i] = publicOutputs[startIndex + i];
        }
        string memory publicOutputFieldValue = StringUtils
            .convertPackedBytesToString(
                packed_field,
                packSize * field_len,
                packSize
            );
        return
            string.concat(
                '"',
                publicOutputFieldName,
                '":"',
                publicOutputFieldValue,
                '"'
            );
    }

    // The proof to this fork test does not include a commitment to the owner address in the public outputs, so this check is commented out
    function validateOwner(
        uint256[14] memory /* publicOutputs */,
        uint256 /* toAddressStartIndex */,
        address /* to */
    ) internal pure {
        // uint256[] memory packed_address = new uint256[](address_len);
        // for (uint256 i = 0; i < address_len; i++) {
        //     packed_address[i] = publicOutputs[toAddressStartIndex + i];
        // }
        // string memory toAddressString = StringUtils.convertPackedBytesToString(
        //     packed_address,
        //     packSize * address_len,
        //     packSize
        // );
        // if (Strings.parseAddress(toAddressString) != to) {
        //     revert OwnerNotInProof();
        // }
    }
}
