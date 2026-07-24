// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import { IDKIMRegistry } from "../../../src/interfaces/IDKIMRegistry.sol";
import { ITestBlueprintGroth16Verifier } from "./ITestBlueprintGroth16Verifier.sol";
import { IZKEmailVerifier } from "../../../src/interfaces/IZKEmailVerifier.sol";

/**
 * @title TestBlueprintZKEmailVerifier
 * @notice A byte-for-byte copy (only the contract name and Groth16 verifier interface type changed)
 * of a real, per-blueprint-generated `ZKEmailVerifier.sol` (sender domain x.com; see
 * parameters.json in this directory for the full blueprint configuration). It implements the real,
 * committed {IZKEmailVerifier} interface -- nothing about verify()'s logic is changed here.
 * @dev This is a test fixture, not a per-blueprint generated artifact: it is committed on purpose so it
 * can be reviewed directly. See ../../../../kusama-grant/milestone-2/03_verifier_interface_and_wrappers.md
 * for how the production `ZKEmailVerifier.sol` is generated per blueprint (and stays gitignored), and
 * README.md in this directory for full provenance of this fixture and the real proof it's tested against.
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
