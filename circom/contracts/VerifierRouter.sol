// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @title Verifier Information Structure
/// @notice Stores details about registered ZK verifier contracts
struct VerifierInfo {
    address verifierAddress;
    bytes4 functionSelector;
}

/// @title ZK Verifier Router Contract
/// @notice Routes verification requests to appropriate registered verifier contracts
/// @dev Acts as a central registry and gateway for multiple ZK verification contracts
contract VerifierRouter is Ownable {
    error InvalidVerifierAddress(address verifier);
    error VerifierAlreadyRegistered(uint256 verifierId);
    error VerifierNotRegistered(uint256 verifierId);
    error VerificationFailed();

    event VerifierRegistered(uint256 indexed id, address indexed verifier, bytes4 functionSelector);
    event VerifierUpdated(
        uint256 indexed id, address indexed oldVerifier, address indexed newVerifier, bytes4 functionSelector
    );
    event VerifierUnregistered(uint256 indexed id);

    mapping(uint256 => VerifierInfo) public verifierRegistry;

    constructor(address initialOwner) Ownable(initialOwner) {}

    /// @notice Registers a new verifier contract
    /// @param verifierId Unique identifier for the verifier
    /// @param verifierAddress Address of the verifier contract
    /// @param functionSelector Function selector for the verification method
    function registerVerifier(uint256 verifierId, address verifierAddress, bytes4 functionSelector)
        external
        onlyOwner
    {
        if (verifierAddress == address(0)) revert InvalidVerifierAddress(verifierAddress);
        if (verifierRegistry[verifierId].verifierAddress != address(0)) {
            revert VerifierAlreadyRegistered(verifierId);
        }

        verifierRegistry[verifierId] = VerifierInfo(verifierAddress, functionSelector);
        emit VerifierRegistered(verifierId, verifierAddress, functionSelector);
    }

    /// @notice Updates an existing verifier contract
    /// @param verifierId Identifier of the verifier to update
    /// @param verifierAddress New address of the verifier contract
    /// @param functionSelector New function selector for the verification method
    function updateVerifier(uint256 verifierId, address verifierAddress, bytes4 functionSelector) external onlyOwner {
        if (verifierRegistry[verifierId].verifierAddress == address(0)) revert VerifierNotRegistered(verifierId);

        address oldVerifier = verifierRegistry[verifierId].verifierAddress;
        verifierRegistry[verifierId] = VerifierInfo(verifierAddress, functionSelector);
        emit VerifierUpdated(verifierId, oldVerifier, verifierAddress, functionSelector);
    }

    /// @notice Unregisters a verifier
    /// @param verifierId Identifier of the verifier to remove
    function unregisterVerifier(uint256 verifierId) external onlyOwner {
        if (verifierRegistry[verifierId].verifierAddress == address(0)) revert VerifierNotRegistered(verifierId);
        // Unchecked block to save gas since there's no risk of underflow/overflow when deleting
        unchecked {
            delete verifierRegistry[verifierId];
        }
        emit VerifierUnregistered(verifierId);
    }

    /// @notice Checks if a verifier is registered
    /// @param verifierId Identifier of the verifier to check
    /// @return isRegistered Whether the verifier is registered
    function isVerifierRegistered(uint256 verifierId) external view returns (bool) {
        return verifierRegistry[verifierId].verifierAddress != address(0);
    }

    /// @notice Retrieves verifier information by ID.
    /// @param verifierId The ID of the verifier.
    /// @return The verifier information.
    function getVerifierInfo(uint256 verifierId) external view returns (VerifierInfo memory) {
        if (verifierRegistry[verifierId].verifierAddress == address(0)) revert VerifierNotRegistered(verifierId);

        return verifierRegistry[verifierId];
    }

    /// @notice Verifies a proof using the specified verifier
    /// @param verifierId The ID of the verifier to use
    /// @param a First part of the ZK proof
    /// @param b Second part of the ZK proof
    /// @param c Third part of the ZK proof
    /// @param publicOutputs Public inputs to the ZK circuit
    function verify(
        uint256 verifierId,
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[] calldata publicOutputs
    ) external view {
        VerifierInfo memory info = verifierRegistry[verifierId];
        if (info.verifierAddress == address(0)) revert VerifierNotRegistered(verifierId);

        bytes memory data = abi.encodePacked(info.functionSelector, a, b, c, publicOutputs);

        (bool success,) = info.verifierAddress.staticcall(data);
        if (!success) revert VerificationFailed();
    }
}
