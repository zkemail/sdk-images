// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {VerifierRouterBaseTest} from "./_VerifierRouterBase.t.sol";
import {VerifierRouter} from "../../../contracts/VerifierRouter.sol";

contract VerifyTest is VerifierRouterBaseTest {
    uint256[2] a;
    uint256[2][2] b;
    uint256[2] c;
    uint256[16] publicOutputsFixedSize;
    uint256[] publicOutputs = new uint256[](16);

    function setUp() public override {
        super.setUp();

        a = [vm.randomUint(), vm.randomUint()];
        b = [[vm.randomUint(), vm.randomUint()], [vm.randomUint(), vm.randomUint()]];
        c = [vm.randomUint(), vm.randomUint()];

        for (uint256 i = 0; i < 16; i++) {
            publicOutputsFixedSize[i] = vm.randomUint();
            publicOutputs[i] = publicOutputsFixedSize[i];
        }
    }

    function test_RevertWhen_VerifierNotRegistered() public {
        verifierId = verifierId + 9;

        vm.expectRevert(abi.encodeWithSelector(VerifierRouter.VerifierNotRegistered.selector, verifierId));
        router.verify(verifierId, a, b, c, publicOutputs);
    }

    function test_RevertWhen_VerifierRevert() public {
        vm.mockCallRevert(verifierAddress, verifySelector, vm.randomBytes(88));

        vm.expectRevert(VerifierRouter.VerificationFailed.selector);
        router.verify(verifierId, a, b, c, publicOutputs);
    }

    function test_SuccessWhen_VerifierSuccess() public {
        vm.mockCall(verifierAddress, verifySelector, "");

        vm.expectCall(verifierAddress, abi.encodeWithSelector(verifySelector, a, b, c, publicOutputsFixedSize));
        router.verify(verifierId, a, b, c, publicOutputs);
    }
}
