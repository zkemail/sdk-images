// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../../src/ZKEmailProof.sol";

contract ZKEmailProofTest is Test {
    ZKEmailProof zkEmailProof;
    address owner = address(0x1);
    address verifier1 = address(0x2);
    address verifier2 = address(0x3);
    address user = address(0x4);

    function setUp() public {
        vm.startPrank(owner);
        zkEmailProof = new ZKEmailProof(owner);
        zkEmailProof.addVerifier(verifier1);
        vm.stopPrank();
    }

    function testOnlyVerifierCanMint() public {
        // Try minting from verifier1 (should succeed)
        vm.startPrank(verifier1);

        Proof memory proof = Proof({
            a: [uint256(1), uint256(2)],
            b: [[uint256(3), uint256(4)], [uint256(5), uint256(6)]],
            c: [uint256(7), uint256(8)]
        });

        uint256[] memory publicOutputs = new uint256[](1); // Initialize publicOutputs as an array with one element
        publicOutputs[0] = uint256(uint160(user)); // publicOutputs[0] is user address

        zkEmailProof.safeMint(
            user,
            1, // blueprintId
            verifier1,
            proof,
            publicOutputs,
            '"email":"user@example.com", "name": "test"', // decodedPublicOutputs
            0
        );

        vm.stopPrank();

        // Check that the token was minted to the user
        assertEq(zkEmailProof.balanceOf(user), 1);

        // Retrieve and display the token URI
        string memory tokenURI = zkEmailProof.tokenURI(0);
        console.log("Token URI: ", tokenURI);
    }

    function testNonVerifierCannotMint() public {
        // Try minting from an unauthorized address (should fail)
        vm.startPrank(address(0x5)); // Unauthorized address

        Proof memory proof = Proof({
            a: [uint256(1), uint256(2)],
            b: [[uint256(3), uint256(4)], [uint256(5), uint256(6)]],
            c: [uint256(7), uint256(8)]
        });

        uint256[] memory publicOutputs = new uint256[](1); // Initialize publicOutputs as an array with one element
        publicOutputs[0] = uint256(uint160(user)); // publicOutputs[0] is user address

        vm.expectRevert(ZKEmailProof.OnlyVerifier.selector);
        zkEmailProof.safeMint(
            user,
            1, // blueprintId
            verifier1,
            proof,
            publicOutputs,
            '"email":"user@example.com", "name": "test"', // decodedPublicOutputs
            0
        );

        vm.stopPrank();
    }

    function testCannotTransferNFT() public {
        // Mint NFT to user
        vm.startPrank(verifier1);

        Proof memory proof = Proof({
            a: [uint256(1), uint256(2)],
            b: [[uint256(3), uint256(4)], [uint256(5), uint256(6)]],
            c: [uint256(7), uint256(8)]
        });

        uint256[] memory publicOutputs = new uint256[](1); // Initialize publicOutputs as an array with one element
        publicOutputs[0] = uint256(uint160(user)); // publicOutputs[0] is user address

        zkEmailProof.safeMint(
            user,
            1, // blueprintId
            verifier1,
            proof,
            publicOutputs,
            '"email":"user@example.com", "name": "test"', // decodedPublicOutputs
            0
        );

        vm.stopPrank();

        // Try transferring NFT (should fail)
        vm.startPrank(user);

        vm.expectRevert(ZKEmailProof.CannotTransferSoulboundToken.selector);
        zkEmailProof.transferFrom(user, address(0x6), 0);

        vm.stopPrank();

        // Retrieve and display the token URI
        string memory tokenURI = zkEmailProof.tokenURI(0);
        console.log("Token URI: ", tokenURI);
    }

    function testAddAndRemoveVerifier() public {
        // Only owner can add verifier
        vm.startPrank(owner);
        zkEmailProof.addVerifier(verifier2);
        vm.stopPrank();

        // Verifier2 should be able to mint now
        vm.startPrank(verifier2);

        Proof memory proof = Proof({
            a: [uint256(10), uint256(20)],
            b: [[uint256(30), uint256(40)], [uint256(50), uint256(60)]],
            c: [uint256(70), uint256(80)]
        });

        uint256[] memory publicOutputs = new uint256[](1); // Initialize publicOutputs as an array with one element
        publicOutputs[0] = uint256(uint160(user)); // publicOutputs[0] is user address

        zkEmailProof.safeMint(
            user,
            2, // blueprintId
            verifier2,
            proof,
            publicOutputs,
            '"email":"user@example.com", "name": "test"', // decodedPublicOutputs
            0
        );

        vm.stopPrank();

        // Retrieve and display the token URI
        string memory tokenURI = zkEmailProof.tokenURI(0);
        console.log("Token URI: ", tokenURI);

        // Remove verifier2
        vm.startPrank(owner);
        zkEmailProof.removeVerifier(verifier2);
        vm.stopPrank();

        // Verifier2 should not be able to mint anymore
        vm.startPrank(verifier2);

        vm.expectRevert(ZKEmailProof.OnlyVerifier.selector);
        zkEmailProof.safeMint(
            user,
            3, // blueprintId
            verifier2,
            proof,
            publicOutputs,
            '"email":"user@example.com", "name": "test"', // decodedPublicOutputs
            0
        );

        vm.stopPrank();
    }
}