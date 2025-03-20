// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {VerifierRouterBaseTest} from "./_VerifierRouterBase.t.sol";

contract RegisterVerifierTest is VerifierRouterBaseTest {
    function test_NotRegistered() public {
        verifierId = vm.randomUint();

        bool isRegistered = router.isVerifierRegistered(verifierId);
        assertFalse(isRegistered);
    }

    function test_Registered() public view {
        bool isRegistered = router.isVerifierRegistered(verifierId);
        assertTrue(isRegistered);
    }
}
