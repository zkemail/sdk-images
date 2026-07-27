// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import { IDKIMRegistry } from "../interfaces/IDKIMRegistry.sol";
import { ITestBlueprintGroth16Verifier } from "./ITestBlueprintGroth16Verifier.sol";
import { IZKEmailVerifier } from "../interfaces/IZKEmailVerifier.sol";

/**
 * @title TestBlueprintZKEmailVerifier
 * @notice A byte-for-byte copy (only the relative import paths changed to match this directory)
 * of the committed test fixture at test/fixtures/testBlueprint/TestBlueprintZKEmailVerifier.sol
 * (sender domain x.com), deployed here to Paseo against the real, already-live DKIMRegistry from
 * milestone 1 -- see ../../../kusama-grant/milestone-2/06_e2e_demo.md for the deployment record and
 * bytecode-identity proof against the original fixture, and the fixture's own README.md for full
 * provenance of the contract and the real proof it's tested against.
 * @dev Not a per-blueprint generated artifact -- see
 * ../../../kusama-grant/milestone-2/03_verifier_interface_and_wrappers.md for how the production
 * `ZKEmailVerifier.sol` is generated per blueprint (and stays gitignored).
 */
contract TestBlueprintZKEmailVerifier is IZKEmailVerifier {
    IDKIMRegistry public immutable DKIM_REGISTRY;
    ITestBlueprintGroth16Verifier public immutable GROTH16_VERIFIER;

    uint256 public constant PUBLIC_KEY_HASH_OFFSET = 0;
    uint256 public constant PUBLIC_INPUTS_LENGTH = 7;
    bytes32 public constant DOMAIN_HASH = keccak256(bytes("x.com"));

    constructor(IDKIMRegistry _dkimRegistry, ITestBlueprintGroth16Verifier _groth16Verifier) {
        if (address(_dkimRegistry) == address(0)) revert InvalidDKIMRegistry();
        if (address(_groth16Verifier) == address(0)) revert InvalidProofVerifier();
        DKIM_REGISTRY = _dkimRegistry;
        GROTH16_VERIFIER = _groth16Verifier;
    }

    /**
     * @inheritdoc IZKEmailVerifier
     */
    function verify(bytes calldata proof, bytes32[] calldata publicInputs) external view {
        if (publicInputs.length != PUBLIC_INPUTS_LENGTH) {
            revert InvalidPublicInputsLength(PUBLIC_INPUTS_LENGTH, publicInputs.length);
        }
        if (!DKIM_REGISTRY.isKeyHashValid(DOMAIN_HASH, publicInputs[PUBLIC_KEY_HASH_OFFSET])) {
            revert InvalidPublicKey();
        }

        (uint256[2] memory pA, uint256[2][2] memory pB, uint256[2] memory pC) =
            abi.decode(proof, (uint256[2], uint256[2][2], uint256[2]));

        uint256[PUBLIC_INPUTS_LENGTH] memory pubSignals;
        for (uint256 i = 0; i < PUBLIC_INPUTS_LENGTH; i++) {
            pubSignals[i] = uint256(publicInputs[i]);
        }

        if (!GROTH16_VERIFIER.verifyProof(pA, pB, pC, pubSignals)) revert InvalidProof();
    }
}
