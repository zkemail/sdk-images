// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {VerifierRouterBaseTest} from "./_VerifierRouterBase.t.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {TestVerifier} from "../../../contracts/test/TestVerifier.sol";
import {VerifierRouter} from "../../../contracts/VerifierRouter.sol";

contract RegisterVerifierTest is VerifierRouterBaseTest {
    address newVerifierAddress;

    function setUp() public override {
        super.setUp();
        newVerifierAddress = vm.randomAddress();
    }

    function test_RegisterVerifier_RevertWhen_NotOwner() public {
        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        router.updateVerifier(verifierId, newVerifierAddress, verifySelector);
    }

    function test_RevertWhen_VerifierNotRegistered() public {
        verifierId = vm.randomUint();

        vm.expectRevert(abi.encodeWithSelector(VerifierRouter.VerifierNotRegistered.selector, verifierId));
        router.updateVerifier(verifierId, newVerifierAddress, verifySelector);
    }

    function test_VerifierUpdatedEvent() public {
        vm.expectEmit();
        emit VerifierRouter.VerifierUpdated(verifierId, verifierAddress, newVerifierAddress, verifySelector);
        router.updateVerifier(verifierId, newVerifierAddress, verifySelector);
    }
}
