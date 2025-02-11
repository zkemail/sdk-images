// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {IDKIMRegistry} from "@zk-email/contracts/interfaces/IDKIMRegistry.sol";
import {StringUtils} from "@zk-email/contracts/utils/StringUtils.sol";
import {IProofOfTwitter_Groth16Verifier} from "../interfaces/IProofOfTwitter_Groth16Verifier.sol";
import {ZKEmailProof, Proof} from "../ZKEmailProof.sol";

/**
 * @title ProofOfTwitter_Verifier
 * @notice Test verifier - "This blueprint is for proving that you owned a twitter handle"
 * https://registry.zk.email/0935faed-002d-4b94-8cbf-476b3b05d9a6/proofs/67058735-af80-4fd8-9188-4f1dcf6fa313?emailProofInfo=%7B%22status%22%3A2%2C%22id%22%3A%2267058735-af80-4fd8-9188-4f1dcf6fa313%22%2C%22blueprintId%22%3A%220935faed-002d-4b94-8cbf-476b3b05d9a6%22%2C%22proofData%22%3A%7B%22pi_a%22%3A%5B%2219014357250828634823182307903338300175090724504730053381041748437315271534735%22%2C%224696736584263604331558996548418964379075298974995081308895901094664013414220%22%2C%221%22%5D%2C%22pi_b%22%3A%5B%5B%2214921817661383102514061633663737655278513086830555952892732430740252347498809%22%2C%222166854163583550883937146753361462030743291876364316237947986608198907783650%22%5D%2C%5B%223006462367602337020834301360939705755022854337806187866519397082844272003175%22%2C%2211235379133743098261599914832460495433729372576618727742968307957676590058492%22%5D%2C%5B%221%22%2C%220%22%5D%5D%2C%22pi_c%22%3A%5B%2212074728462639618962329481847660153022131782339938453563695793425878404386242%22%2C%226191198095661398861156257556629402133673552135534202763283703513866655390104%22%2C%221%22%5D%2C%22protocol%22%3A%22groth16%22%7D%2C%22publicData%22%3A%7B%22handle%22%3A%5B%22john_guilding%22%5D%7D%2C%22externalInputs%22%3A%7B%22address%22%3A%220x91AdDB0E8443C83bAf2aDa6B8157B38f814F0bcC%22%7D%2C%22startedAt%22%3A%222025-02-11T15%3A35%3A03.000Z%22%2C%22provedAt%22%3A%222025-02-11T15%3A35%3A49.000Z%22%2C%22isLocal%22%3Afalse%7D
 */
contract ProofOfTwitter_Verifier {
    address public immutable dkimRegistry;
    address public immutable verifier;
    address public immutable zkEmailProof;

    uint16 public constant packSize = 31;
    string public constant domain = "x.com";
    uint16 public constant handle_len = 3;
    uint16 public constant address_len = 3;

    error InvalidDKIMPublicKeyHash();
    error InvalidProof();

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
        uint[2] calldata a,
        uint[2][2] calldata b,
        uint[2] calldata c,
        uint[8] calldata publicOutputs
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
            !IProofOfTwitter_Groth16Verifier(verifier).verifyProof(
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
        uint256[8] calldata publicOutputs,
        string[1] calldata publicOutputFieldNames,
        address to,
        uint256 blueprintId,
        uint256 toAddressIndex
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
            !IProofOfTwitter_Groth16Verifier(verifier).verifyProof(
                a,
                b,
                c,
                publicOutputs
            )
        ) revert InvalidProof();

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
            decodedPublicOutputs,
            toAddressIndex
        );
    }

    function decodePublicOutputs(
        string[1] calldata publicOutputFieldNames,
        uint256[8] calldata publicOutputs
    ) internal pure returns (string memory) {
        uint256 startIndex = 1;
        string memory handle_string = convertPackedFieldsToPublicOutput(
            publicOutputs,
            startIndex,
            publicOutputFieldNames[0],
            handle_len
        );

        return string.concat("{", handle_string, "}");
    }

    function convertPackedFieldsToPublicOutput(
        uint256[8] memory publicOutputs,
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
}
