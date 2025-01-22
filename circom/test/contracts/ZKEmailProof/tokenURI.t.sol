// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {DKIMRegistry} from "@zk-email/contracts/DKIMRegistry.sol";
import {ZKEmailProof, Proof, ZKEmailProofMetadata} from "../../../contracts/ZKEmailProof.sol";
import {MockVerifier} from "../../../contracts/test/MockVerifier.sol";

contract ZKEmailProof_TokenURI_Test is Test {
    using Strings for uint256;
    using Strings for address;

    ZKEmailProof zkEmailProof;
    address owner = address(0x1);
    address dkimRegistry = address(new DKIMRegistry(owner));
    address verifier = address(new MockVerifier());
    address user = address(0x5);
    string domainName = "gmail.com";
    bytes32 publicKeyHash =
        0x0ea9c777dc7110e5a9e89b13f0cfc540e3845ba120b2b6dc24024d61488d4788;

    address alice = address(0x2);
    uint256 blueprintId = 1;

    function setUp() public {
        vm.startPrank(owner);
        zkEmailProof = new ZKEmailProof(owner, dkimRegistry);
        zkEmailProof.addVerifier(verifier);

        DKIMRegistry(dkimRegistry).setDKIMPublicKeyHash(
            domainName,
            publicKeyHash
        );
        vm.stopPrank();
    }

    function test_ZKEmailProof_TokenURI() public {
        Proof memory proof = Proof({
            a: [uint256(1), uint256(2)],
            b: [[uint256(3), uint256(4)], [uint256(5), uint256(6)]],
            c: [uint256(7), uint256(8)]
        });

        uint256[] memory publicOutputs = new uint256[](1); // Initialize publicOutputs as an array with one element
        publicOutputs[0] = uint256(uint160(alice)); // publicOutputs[0] is user address
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
            0
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
