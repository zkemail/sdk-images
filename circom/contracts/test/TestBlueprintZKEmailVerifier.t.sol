// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

import { Test } from "forge-std/Test.sol";
import { DKIMRegistryMock } from "./DKIMRegistryMock.sol";
import { TestBlueprintGroth16Verifier } from "./fixtures/testBlueprint/TestBlueprintGroth16Verifier.sol";
import { TestBlueprintZKEmailVerifier } from "./fixtures/testBlueprint/TestBlueprintZKEmailVerifier.sol";
import { ITestBlueprintGroth16Verifier } from "./fixtures/testBlueprint/ITestBlueprintGroth16Verifier.sol";
import { TestBlueprintFixture } from "./fixtures/testBlueprint/TestBlueprintFixture.sol";
import { IZKEmailVerifier } from "../src/interfaces/IZKEmailVerifier.sol";

/**
 * @notice Exercises ZKEmailVerifier's proof-decode/DKIM-gate plumbing against a real (non-mock)
 * Groth16 verifier and a real proof for a real, already-deployed blueprint (see
 * test/fixtures/testBlueprint/README.md for provenance), rather than MockGroth16Verifier's
 * always-`true` stub.
 */
contract TestBlueprintZKEmailVerifierTest is Test {
    DKIMRegistryMock internal dkimRegistry;
    TestBlueprintGroth16Verifier internal groth16Verifier;
    TestBlueprintZKEmailVerifier internal verifier;

    function setUp() public {
        dkimRegistry = new DKIMRegistryMock();
        groth16Verifier = new TestBlueprintGroth16Verifier();
        verifier =
            new TestBlueprintZKEmailVerifier(dkimRegistry, ITestBlueprintGroth16Verifier(address(groth16Verifier)));
    }

    function test_verify_passesForRealProof() public view {
        bytes memory proof = abi.encode(TestBlueprintFixture.pA(), TestBlueprintFixture.pB(), TestBlueprintFixture.pC());
        verifier.verify(proof, TestBlueprintFixture.publicInputs());
    }

    function test_verify_revertsWhen_publicInputTampered() public {
        bytes memory proof = abi.encode(TestBlueprintFixture.pA(), TestBlueprintFixture.pB(), TestBlueprintFixture.pC());
        bytes32[] memory publicInputs = TestBlueprintFixture.publicInputs();
        // Flip a public signal unrelated to the (mocked) DKIM-gate check; the real Groth16
        // verifier must reject the now-mismatched proof/public-input pair.
        publicInputs[1] = bytes32(uint256(publicInputs[1]) ^ 1);

        vm.expectRevert(IZKEmailVerifier.InvalidProof.selector);
        verifier.verify(proof, publicInputs);
    }

    function test_verify_revertsWhen_dkimKeyHashInvalid() public {
        dkimRegistry.setShouldValidate(false);
        bytes memory proof = abi.encode(TestBlueprintFixture.pA(), TestBlueprintFixture.pB(), TestBlueprintFixture.pC());

        vm.expectRevert(IZKEmailVerifier.InvalidPublicKey.selector);
        verifier.verify(proof, TestBlueprintFixture.publicInputs());
    }
}
