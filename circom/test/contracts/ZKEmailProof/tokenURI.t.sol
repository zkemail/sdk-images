// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {Proof} from "../../../contracts/ZKEmailProof.sol";
import {BaseTest} from "./BaseTest.t.sol";

contract ZKEmailProof_TokenURI_Test is BaseTest {
    using Strings for uint256;
    using Strings for address;

    function setUp() public override {
        super.setUp();
    }

    function test_ZKEmailProof_TokenURI() public {
        string memory decodedPublicOutputs = '"to":2,"username":"John Smith"';

        vm.prank(address(verifier));
        zkEmailProof.mintProof(
            alice,
            blueprintId,
            address(verifier),
            domainName,
            publicKeyHash,
            proof,
            publicOutputs,
            decodedPublicOutputs,
            proverEthAddressIdx
        );

        uint256 tokenId = 0;
        string memory tokenUri = zkEmailProof.tokenURI(tokenId);

        string memory expectedJson = string.concat(
            '{"name":"ZKEmail Proof #0","description":"Soulbound NFT representing a valid ZK Email proof for an account","attributes":[{"trait_type":"Blueprint ID","value":"1"},{"trait_type":"Proof_a","value":[1,2]},{"trait_type":"Proof_b","value":[[3,4],[5,6]]},{"trait_type":"Proof_c","value":[7,8]},{"trait_type":"Public Outputs","value":"[2]"},{"trait_type":"Decoded Public Outputs","value":{"to":2,"username":"John Smith"}},{"trait_type":"Verifier","value":"',
            address(verifier).toHexString(),
            '"}]}'
        );
        string memory expectedTokenUri = string(
            abi.encodePacked(
                "data:application/json;base64,",
                Base64.encode(bytes(expectedJson))
            )
        );

        assertEq(tokenUri, expectedTokenUri);
    }
}
