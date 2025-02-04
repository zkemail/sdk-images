// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {DKIMRegistry} from "@zk-email/contracts/DKIMRegistry.sol";
import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {IVerifier} from "../../../contracts/interfaces/IVerifier.sol";
import {ZKEmailProof, Proof, ZKEmailProofMetadata} from "../../../contracts/ZKEmailProof.sol";
import {ExtractGoogleDomain_Verifier} from "../../../contracts/test/ExtractGoogleDomain_Verifier.sol";

contract ZKEmailProof_ExtractGoogleDomain_Verifier_Fork_Test is Test {
    address constant DEPLOYED_VERIFIER =
        0x7019c2E274c77dd6E9e4C2707068BC6e690eA0AF;

    address public owner;
    address public alice;

    DKIMRegistry dkimRegistry;
    IVerifier groth16Verifier;
    ZKEmailProof zkEmailProof;
    ExtractGoogleDomain_Verifier verifier;

    Proof proof;
    uint256[5] publicOutputs;
    string[1] publicOutputFieldNames;
    address to;
    uint256 blueprintId;
    uint256 toAddressIndex;

    string domainName;
    bytes32 publicKeyHash;

    function setUp() public {
        string memory BASE_SEPOLIA_RPC_URL = vm.envString(
            "BASE_SEPOLIA_RPC_URL"
        );
        vm.createSelectFork(BASE_SEPOLIA_RPC_URL);
        vm.rollFork(20880810);

        owner = address(1);
        // We're setting alice to a value in the publicOutputs array, but this is a bit of a hack as
        // the value is not actually an owner address according to the original proof
        alice = address(2440484440003696966756646629102736908273017697);

        dkimRegistry = new DKIMRegistry(owner);
        groth16Verifier = IVerifier(DEPLOYED_VERIFIER);
        zkEmailProof = new ZKEmailProof(owner);
        verifier = new ExtractGoogleDomain_Verifier(
            address(dkimRegistry),
            address(groth16Verifier),
            address(zkEmailProof)
        );

        proof = Proof({
            a: [
                1692793978230725134718537588656764633251068598376840802181836497833618927933,
                17936084840096216584367612016954721127830087185756579787574184783724508377771
            ],
            b: [
                [
                    19219283647539122059053522276695880879148407165532565741834089795370991358107,
                    12847465177655014214596840354520911080160186515965227558637903538532772737079
                ],
                [
                    4767667169902665979072671086515676224560114043872022242063506932480784453004,
                    4663911819773402879184509610027021038350291289101188966445234717972308766789
                ]
            ],
            c: [
                13913147805600869559156345614958577304807929921893387548191618314601950326296,
                20488551472834533113028258652399137644428184836935659104725497397636885729869
            ]
        });
        publicOutputs = [
            3024598485745563149860456768272954250618223591034926533254923041921841324429,
            2440484440003696966756646629102736908273017697,
            0,
            0,
            0
        ];
        publicOutputFieldNames = ["sender_domain"];
        to = alice;
        blueprintId = 1;
        toAddressIndex = 1;

        domainName = "accounts.google.com";
        publicKeyHash = bytes32(publicOutputs[0]);

        vm.startPrank(owner);
        zkEmailProof.addVerifier(address(verifier));
        dkimRegistry.setDKIMPublicKeyHash(domainName, publicKeyHash);
        vm.stopPrank();
    }

    function test_Verify() public view {
        verifier.verify(proof.a, proof.b, proof.c, publicOutputs);
    }

    function test_VerifyAndMint() public {
        string
            memory expectedDecodedPublicOutputs = '{"sender_domain":"accounts.google.com"}';

        verifier.verifyAndMint(
            proof.a,
            proof.b,
            proof.c,
            publicOutputs,
            publicOutputFieldNames,
            to,
            blueprintId,
            toAddressIndex
        );

        uint256 tokenId = 0;
        assertEq(zkEmailProof.balanceOf(alice), 1);
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
        assertEq(metadata.publicOutputs[0], publicOutputs[0]);
        assertEq(metadata.publicOutputs[1], publicOutputs[1]);
        assertEq(metadata.publicOutputs[2], publicOutputs[2]);
        assertEq(metadata.publicOutputs[3], publicOutputs[3]);
        assertEq(metadata.publicOutputs[4], publicOutputs[4]);
        assertEq(metadata.decodedPublicOutputs, expectedDecodedPublicOutputs);
    }
}
