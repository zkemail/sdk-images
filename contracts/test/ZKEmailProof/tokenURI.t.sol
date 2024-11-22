// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {ZKEmailProof, ZKEmailProofMetadata} from "../../src/ZKEmailProof.sol";

contract ZKEmailProof_TokenURI_Test is Test {
    using Strings for uint256;
    using Strings for address;

    ZKEmailProof public zkEmailProof;
    address public alice;
    address public bob;

    function setUp() public {
        zkEmailProof = new ZKEmailProof();
        alice = address(1);
        bob = address(2);
    }

    function test_ZKEmailProof_TokenURI() public {
        uint256 blueprintId = 1;
        uint256[] memory proof = new uint256[](2);
        proof[0] = uint256(uint160(alice));
        proof[1] = 42;
        address verifier = address(3);
        string memory publicOutputs = "Test Public Outputs";

        vm.prank(alice);
        zkEmailProof.safeMint(
            alice,
            blueprintId,
            proof,
            verifier,
            publicOutputs
        );

        uint256 tokenId = 0;
        string memory tokenUri = zkEmailProof.tokenURI(tokenId);

        string
            memory expectedJson = '{"name": "ZKEmailProof NFT #0","description": "Soulbound NFT representing a valid ZK Email proof for an account","attributes": [{ "trait_type": "Blueprint ID", "value": "1" },{ "trait_type": "Verifier", "value": "0x0000000000000000000000000000000000000003" },{ "trait_type": "Public Outputs", "value": "Test Public Outputs" },{ "trait_type": "Proof", "value": [1,42] }]}';
        string memory expectedTokenUri = string(
            abi.encodePacked(
                "data:application/json;base64,",
                Base64.encode(bytes(expectedJson))
            )
        );

        assertEq(tokenUri, expectedTokenUri);
    }
}
