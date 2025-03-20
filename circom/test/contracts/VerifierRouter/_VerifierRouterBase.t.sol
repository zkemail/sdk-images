// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Test} from "forge-std/Test.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {TestVerifier} from "../../../contracts/test/TestVerifier.sol";
import {VerifierRouter} from "../../../contracts/VerifierRouter.sol";

contract VerifierRouterBaseTest is Test {
    address alice;
    VerifierRouter router;
    uint256 verifierId;
    address verifierAddress;
    bytes4 verifySelector;

    function setUp() public virtual {
        alice = vm.randomAddress();
        verifierId = vm.randomUint();
        verifierAddress = vm.randomAddress();
        verifySelector = TestVerifier.verify.selector;

        address owner = address(this);
        router = new VerifierRouter(owner);

        router.registerVerifier(verifierId, verifierAddress, verifySelector);
    }
}
