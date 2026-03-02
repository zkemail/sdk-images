// SPDX-License-Identifier: MIT
pragma solidity ^0.8.34;

import { IDKIMRegistry } from "@zk-email/contracts/interfaces/IERC7969.sol";

contract DKIMRegistryMock is IDKIMRegistry {
    bool public shouldValidate = true;

    function setShouldValidate(bool _value) external {
        shouldValidate = _value;
    }

    function isKeyHashValid(bytes32, bytes32) external view override returns (bool) {
        return shouldValidate;
    }
}
