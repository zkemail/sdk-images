// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {DKIMRegistry} from "@zk-email/contracts/DKIMRegistry.sol";
import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {IProofOfDevconRejection_Groth16Verifier} from "../../../contracts/interfaces/IProofOfDevconRejection_Groth16Verifier.sol";
import {ZKEmailProof, Proof, ZKEmailProofMetadata} from "../../../contracts/ZKEmailProof.sol";
import {ProofOfDevconRejection_Verifier} from "../../../contracts/test/ProofOfDevconRejection_Verifier.sol";

contract ZKEmailProof_ProofOfDevconRejection_Verifier_Fork_Test is Test {
    address constant DEPLOYED_VERIFIER =
        0x2747bA8A3036D92114c92502a40c8129bdCaBe54;

    address public owner;
    address public alice;

    DKIMRegistry dkimRegistry;
    IProofOfDevconRejection_Groth16Verifier groth16Verifier;
    ZKEmailProof zkEmailProof;
    ProofOfDevconRejection_Verifier verifier;

    Proof proof;
    uint256[14] publicOutputs;
    string[3] publicOutputFieldNames;
    address to;
    uint256 blueprintId;
    uint256 toAddressStartIndex;

    string domainName;
    bytes32 publicKeyHash;

    function setUp() public {
        string memory SEPOLIA_RPC_URL = vm.envString("SEPOLIA_RPC_URL");
        vm.createSelectFork(SEPOLIA_RPC_URL);

        owner = address(1);
        alice = address(2);

        dkimRegistry = new DKIMRegistry(owner);
        groth16Verifier = IProofOfDevconRejection_Groth16Verifier(
            DEPLOYED_VERIFIER
        );
        zkEmailProof = new ZKEmailProof(owner);
        verifier = new ProofOfDevconRejection_Verifier(
            address(dkimRegistry),
            address(groth16Verifier),
            address(zkEmailProof)
        );

        proof = Proof({
            a: [
                21787286103996958001159036814651275283921884310045619493637386483198756312803,
                2205533941551717183218748965605497367819929589941304242492909336124299056793
            ],
            b: [
                [
                    15711218089500275950782380242514744500211469485416930788764976422281265079884,
                    16368380148416915391563570676054658378521066661349226974852210657544444151676
                ],
                [
                    8102132229674412045503895124318152233653700236174801369606001768756818831626,
                    5111427434321208849913125573505169044530474322393548934654702436226987268363
                ]
            ],
            c: [
                15810970334095572684469054695018586972286456935909780708255922956073629053888,
                16659541395365721697047353511388989965567573573153580998891853733989213312666
            ]
        });
        publicOutputs = [
            8011766048918436304234337347171138895102985966651471271518887910697337713809,
            1852337994,
            0,
            0,
            57377328031630107749936499991080080299453129889983133523939340367986255164,
            5438187003054578043726741043588292439992695,
            0,
            0,
            0,
            0,
            0,
            203411379827238570491176173578436868093537450257881964775793530671143609719,
            2037169922858342507125,
            0
        ];
        publicOutputFieldNames = [
            "recipient_name",
            "proposal_title",
            "rejection_line"
        ];
        to = alice;
        blueprintId = 1;
        toAddressStartIndex = 1;

        domainName = "devcon.org";
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
            memory expectedDecodedPublicOutputs = '{"recipient_name":"John","proposal_title":"<em>Making Smart Accounts easy with ZK Email</em>","rejection_line":"we were unable to accept this submission"}';

        verifier.verifyAndMint(
            proof.a,
            proof.b,
            proof.c,
            publicOutputs,
            publicOutputFieldNames,
            to,
            blueprintId,
            toAddressStartIndex
        );

        uint256 tokenId = 0;
        assertEq(zkEmailProof.balanceOf(alice), 1);
        assertEq(zkEmailProof.ownerOf(tokenId), alice);

        ZKEmailProofMetadata memory metadata = zkEmailProof.getMetadata(alice);
        assertEq(metadata.blueprintId, blueprintId);
        assertEq(metadata.verifier, address(verifier));
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
        assertEq(metadata.publicOutputs[5], publicOutputs[5]);
        assertEq(metadata.publicOutputs[6], publicOutputs[6]);
        assertEq(metadata.publicOutputs[7], publicOutputs[7]);
        assertEq(metadata.publicOutputs[8], publicOutputs[8]);
        assertEq(metadata.publicOutputs[9], publicOutputs[9]);
        assertEq(metadata.publicOutputs[10], publicOutputs[10]);
        assertEq(metadata.publicOutputs[11], publicOutputs[11]);
        assertEq(metadata.publicOutputs[12], publicOutputs[12]);
        assertEq(metadata.publicOutputs[13], publicOutputs[13]);
        assertEq(metadata.decodedPublicOutputs, expectedDecodedPublicOutputs);
    }
}
