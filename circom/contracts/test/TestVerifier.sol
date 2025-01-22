// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import {IDKIMRegistry} from "@zk-email/contracts/interfaces/IDKIMRegistry.sol";
import {MockGroth16Verifier} from "./MockGroth16Verifier.sol";
import {ZKEmailProof, Proof} from "../ZKEmailProof.sol";

contract TestVerifier {
    address public dkimRegistry;
    address public verifier;
    address public zkEmailProof;

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
        uint[5] calldata signals,
        string memory domain,
        bytes32 publicKeyHash
    ) external view {
        require(
            IDKIMRegistry(dkimRegistry).isDKIMPublicKeyHashValid(
                domain,
                publicKeyHash
            ),
            "RSA public key incorrect"
        );

        require(
            MockGroth16Verifier(verifier).verifyProof(a, b, c, signals),
            "Invalid proof"
        );
    }

    function verifyAndMint(
        uint[2] calldata a,
        uint[2][2] calldata b,
        uint[2] calldata c,
        uint[5] calldata signals,
        string memory domain,
        bytes32 publicKeyHash,
        address to,
        uint256 blueprintId,
        string memory decodedPublicOutputs,
        uint256 toAddressIndex
    ) external {
        require(
            IDKIMRegistry(dkimRegistry).isDKIMPublicKeyHashValid(
                domain,
                publicKeyHash
            ),
            "RSA public key incorrect"
        );

        require(
            MockGroth16Verifier(verifier).verifyProof(a, b, c, signals),
            "Invalid proof"
        );

        Proof memory proof = Proof(a, b, c);

        uint256[] memory dynamicSignals = abi.decode(
            abi.encode(signals),
            (uint256[])
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
}
