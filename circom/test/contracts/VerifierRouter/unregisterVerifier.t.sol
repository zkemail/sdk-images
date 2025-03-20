// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {VerifierRouterBaseTest} from "./_VerifierRouterBase.t.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {VerifierRouter} from "../../../contracts/VerifierRouter.sol";

contract UnregisterVerifierTest is VerifierRouterBaseTest {
    function test_RevertWhen_NotOwner() public {
        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        router.unregisterVerifier(verifierId);
    }

    function test_RevertWhen_VerifierNotRegistered() public {
        verifierId = vm.randomUint();

        vm.expectRevert(abi.encodeWithSelector(VerifierRouter.VerifierNotRegistered.selector, verifierId));
        router.unregisterVerifier(verifierId);
    }

    function test_VerifierUnregisteredEvent() public {
        vm.expectEmit();
        emit VerifierRouter.VerifierUnregistered(verifierId);
        router.unregisterVerifier(verifierId);
    }
}
