// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {DKIMRegistry} from "@zk-email/contracts/DKIMRegistry.sol";
import {ZKEmailProof, Proof, ZKEmailProofMetadata} from "../../../contracts/ZKEmailProof.sol";
import {MockVerifier} from "../../../contracts/test/MockVerifier.sol";

contract ZKEmailProof_SafeMint_Test is Test {
    ZKEmailProof public zkEmailProof;
    address public alice;
    address public bob;
    address public admin;

    uint256 blueprintId;
    Proof proof;
    uint256[] publicOutputs = new uint256[](1);
    string decodedPublicOutputs;
    address verifier;

    address dkimRegistry;
    string domainName = "gmail.com";
    bytes32 publicKeyHash =
        0x0ea9c777dc7110e5a9e89b13f0cfc540e3845ba120b2b6dc24024d61488d4788;

    function setUp() public {
        alice = address(1);
        bob = address(2);
        admin = address(3);
        dkimRegistry = address(new DKIMRegistry(admin));
        zkEmailProof = new ZKEmailProof(admin, dkimRegistry);

        blueprintId = 1;
        proof = Proof({
            a: [uint256(1), uint256(2)],
            b: [[uint256(3), uint256(4)], [uint256(5), uint256(6)]],
            c: [uint256(7), uint256(8)]
        });
        publicOutputs[0] = uint256(uint160(alice));
        decodedPublicOutputs = '"to": blueprintId, "username": "John Smith"';
        verifier = address(new MockVerifier());

        vm.prank(admin);
        DKIMRegistry(dkimRegistry).setDKIMPublicKeyHash(
            domainName,
            publicKeyHash
        );
        // zkEmailProof.setVerifier(address(verifier));
        vm.stopPrank();
    }

    function test_ZKEmailProof_SafeMint_RevertWhen_VerifierNotSet() public {
        ZKEmailProof newZkEmailProof = new ZKEmailProof(admin, dkimRegistry);

        vm.expectRevert(ZKEmailProof.OnlyVerifier.selector);
        // newZkEmailProof.mintProof(
        //     alice,
        //     blueprintId,
        //     proof,
        //     publicOutputs,
        //     decodedPublicOutputs
        // );
        newZkEmailProof.mintProof(
            alice,
            blueprintId,
            verifier,
            domainName,
            publicKeyHash,
            proof,
            publicOutputs,
            decodedPublicOutputs,
            0
        );
    }

    function test_ZKEmailProof_SafeMint_RevertWhen_NotVerifier() public {
        vm.expectRevert(ZKEmailProof.OnlyVerifier.selector);
        // zkEmailProof.mintProof(
        //     alice,
        //     blueprintId,
        //     proof,
        //     publicOutputs,
        //     decodedPublicOutputs
        // );
        zkEmailProof.mintProof(
            alice,
            blueprintId,
            verifier,
            domainName,
            publicKeyHash,
            proof,
            publicOutputs,
            decodedPublicOutputs,
            0
        );
    }

    function test_ZKEmailProof_SafeMint() public {
        vm.prank(admin);
        zkEmailProof.addVerifier(address(verifier));
        vm.stopPrank();

        vm.prank(address(verifier));
        // zkEmailProof.mintProof(
        //     alice,
        //     blueprintId,
        //     proof,
        //     publicOutputs,
        //     decodedPublicOutputs
        // );
        zkEmailProof.mintProof(
            alice,
            blueprintId,
            verifier,
            domainName,
            publicKeyHash,
            proof,
            publicOutputs,
            decodedPublicOutputs,
            0
        );

        uint256 tokenId = 0;
        assertEq(zkEmailProof.ownerOf(tokenId), alice);

        ZKEmailProofMetadata memory metadata = zkEmailProof.getMetadata(alice);
        assertEq(metadata.blueprintId, blueprintId);
        assertEq(metadata.proof.a[0], proof.a[0]);
        assertEq(metadata.proof.a[1], proof.a[1]);
        assertEq(metadata.proof.b[0][0], proof.b[0][0]);
        assertEq(metadata.proof.b[0][1], proof.b[0][1]);
        assertEq(metadata.proof.b[1][0], proof.b[1][0]);
        assertEq(metadata.proof.b[1][1], proof.b[1][1]);
        assertEq(metadata.proof.c[0], proof.c[0]);
        assertEq(metadata.proof.c[1], proof.c[1]);
        assertEq(metadata.publicOutputs, publicOutputs);
        assertEq(metadata.decodedPublicOutputs, decodedPublicOutputs);
    }

    // function test_ZKEmailProof_SafeMint_CalledFromVerifier() public {
    //     verifier.verifyAndMint(proof.a, proof.b, proof.c, publicOutputs);

    //     uint256 tokenId = 0;
    //     assertEq(zkEmailProof.ownerOf(tokenId), alice);

    //     ZKEmailProofMetadata memory metadata = zkEmailProof.getMetadata(alice);
    //     assertEq(metadata.blueprintId, blueprintId);
    //     assertEq(metadata.proof.a[0], proof.a[0]);
    //     assertEq(metadata.proof.a[1], proof.a[1]);
    //     assertEq(metadata.proof.b[0][0], proof.b[0][0]);
    //     assertEq(metadata.proof.b[0][1], proof.b[0][1]);
    //     assertEq(metadata.proof.b[1][0], proof.b[1][0]);
    //     assertEq(metadata.proof.b[1][1], proof.b[1][1]);
    //     assertEq(metadata.proof.c[0], proof.c[0]);
    //     assertEq(metadata.proof.c[1], proof.c[1]);
    //     assertEq(metadata.publicOutputs, publicOutputs);
    //     assertEq(metadata.decodedPublicOutputs, decodedPublicOutputs);
    // }

    function test_ZKEmailProof_SafeMint_RevertWhen_OwnerNotInPublicOutputs()
        public
    {
        publicOutputs[0] = uint256(uint160(bob)); // invalid owner

        vm.prank(admin);
        zkEmailProof.addVerifier(address(verifier));
        vm.stopPrank();

        vm.prank(address(verifier));
        vm.expectRevert(ZKEmailProof.OwnerNotInProof.selector);
        // zkEmailProof.mintProof(
        //     alice,
        //     blueprintId,
        //     proof,
        //     publicOutputs,
        //     decodedPublicOutputs
        // );
        zkEmailProof.mintProof(
            alice,
            blueprintId,
            verifier,
            domainName,
            publicKeyHash,
            proof,
            publicOutputs,
            decodedPublicOutputs,
            0
        );
    }
}
