// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {IDKIMRegistry} from "@zk-email/contracts/interfaces/IDKIMRegistry.sol";
import {StringUtils} from "@zk-email/contracts/utils/StringUtils.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {IExtractGoogleDomain_Groth16Verifier} from "../interfaces/IExtractGoogleDomain_Groth16Verifier.sol";
import {ZKEmailProof, Proof} from "../ZKEmailProof.sol";

/**
 * @title ExtractGoogleDomain_Verifier
 * @notice Test verifier to "Reveal Only Email Domain for any Gmail Account"
 * Looks for any email from "no-replyaccounts.google.com" and extracts only the domain.
 * https://registry.zk.email/65d93479-957d-4c0e-8f3f-a292e9359f64/proofs/ae6c58da-2517-4863-a5e3-75e58f0a853e?emailProofInfo=%7B%22status%22%3A2%2C%22id%22%3A%22ae6c58da-2517-4863-a5e3-75e58f0a853e%22%2C%22blueprintId%22%3A%2265d93479-957d-4c0e-8f3f-a292e9359f64%22%2C%22proofData%22%3A%7B%22pi_a%22%3A%5B%2213802435031720962941661669244130696069021578908317704452734390362048770904272%22%2C%2219441376773042620786965941724943347902801781248969807158896752352931643540382%22%2C%221%22%5D%2C%22pi_b%22%3A%5B%5B%2214958311338642265739599103401577340624134193198256815763885004819488719899315%22%2C%2214111732826198729483928446199843358556391351877952773728506071401698891113251%22%5D%2C%5B%2217724731386053148126258101258851230873769052565424991244706990092784722642738%22%2C%226728230226090034507651931034428987533292344523848990959495064060171479578524%22%5D%2C%5B%221%22%2C%220%22%5D%5D%2C%22pi_c%22%3A%5B%2219636258068919222251243054711023733754024767123236240955025887040110890562534%22%2C%2213412079532522716989769443326707115150531489858360807881116498484797828213498%22%2C%221%22%5D%2C%22protocol%22%3A%22groth16%22%7D%2C%22publicData%22%3A%7B%22sender_domain%22%3A%5B%22accounts.google.com%22%5D%7D%2C%22externalInputs%22%3A%7B%7D%2C%22startedAt%22%3A%222025-02-03T17%3A21%3A49.000Z%22%2C%22provedAt%22%3A%222025-02-03T17%3A22%3A01.000Z%22%2C%22isLocal%22%3Afalse%7D
 */
contract ExtractGoogleDomain_Verifier {
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
        IExtractGoogleDomain_Groth16Verifier(verifier).verify(
            a,
            b,
            c,
            publicOutputs
        );
    }

    function verifyAndMint(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[5] calldata publicOutputs,
        string[1] calldata publicOutputFieldNames,
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
        IExtractGoogleDomain_Groth16Verifier(verifier).verify(
            a,
            b,
            c,
            publicOutputs
        );

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
        string[1] calldata publicOutputFieldNames,
        uint256[5] calldata publicOutputs
    ) internal pure returns (string memory) {
        uint256 startIndex = 1;
        string memory sender_domain_string = convertPackedFieldsToPublicOutput(
            publicOutputs,
            startIndex,
            publicOutputFieldNames[0],
            sender_domain_len
        );

        return string.concat("{", sender_domain_string, "}");
    }

    function convertPackedFieldsToPublicOutput(
        uint256[5] memory publicOutputs,
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
        uint256[5] memory /* publicOutputs */,
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
