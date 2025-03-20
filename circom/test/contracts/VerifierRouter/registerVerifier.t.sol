// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {VerifierRouterBaseTest} from "./_VerifierRouterBase.t.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {VerifierRouter} from "../../../contracts/VerifierRouter.sol";

contract RegisterVerifierTest is VerifierRouterBaseTest {
    uint256 registeredVerifierId;

    function setUp() public override {
        super.setUp();

        registeredVerifierId = verifierId;
        verifierId = vm.randomUint();
    }

    function test_RegisterVerifier_RevertWhen_NotOwner() public {
        vm.prank(alice);
        vm.expectRevert(abi.encodeWithSelector(Ownable.OwnableUnauthorizedAccount.selector, alice));
        router.registerVerifier(verifierId, verifierAddress, verifySelector);
    }

    function test_RevertWhen_InvalidVerifierAddress() public {
        address invalidAddress = address(0);

        vm.expectRevert(abi.encodeWithSelector(VerifierRouter.InvalidVerifierAddress.selector, invalidAddress));
        router.registerVerifier(verifierId, invalidAddress, verifySelector);
    }

    function test_RevertWhen_VerifierAlreadyRegistered() public {
        verifierId = registeredVerifierId;

        vm.expectRevert(abi.encodeWithSelector(VerifierRouter.VerifierAlreadyRegistered.selector, verifierId));
        router.registerVerifier(verifierId, verifierAddress, verifySelector);
    }

    function test_VerifierRegisteredEvent() public {
        vm.expectEmit();
        emit VerifierRouter.VerifierRegistered(verifierId, verifierAddress, verifySelector);
        router.registerVerifier(verifierId, verifierAddress, verifySelector);
    }
}
