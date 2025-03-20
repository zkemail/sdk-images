// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {VerifierRouterBaseTest} from "./_VerifierRouterBase.t.sol";
import {VerifierInfo} from "../../../contracts/VerifierRouter.sol";
import {VerifierRouter} from "../../../contracts/VerifierRouter.sol";

contract RegisterVerifierTest is VerifierRouterBaseTest {
    function test_RevertWhen_VerifierNotRegistered() public {
        verifierId = vm.randomUint();

        vm.expectRevert(abi.encodeWithSelector(VerifierRouter.VerifierNotRegistered.selector, verifierId));
        router.getVerifierInfo(verifierId);
    }

    function test_CorrectResult() public view {
        VerifierInfo memory verifierInfo = router.getVerifierInfo(verifierId);

        assertTrue(verifierInfo.verifierAddress == verifierAddress);
        assertTrue(verifierInfo.functionSelector == verifySelector);
    }
}
