// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {DKIMRegistry} from "@zk-email/contracts/DKIMRegistry.sol";
import {ERC721} from "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import {IProofOfTwitter_Groth16Verifier} from "../../../contracts/interfaces/IProofOfTwitter_Groth16Verifier.sol";
import {ZKEmailProof, Proof, ZKEmailProofMetadata} from "../../../contracts/ZKEmailProof.sol";
import {ProofOfTwitter_Verifier} from "../../../contracts/test/ProofOfTwitter_Verifier.sol";

contract ZKEmailProof_ProofOfTwitter_Verifier_Fork_Test is Test {
    address constant DEPLOYED_VERIFIER =
        0xe4Cab1425E02FF5Ae59fdD8a4e90c1F5b05C4164;

    address public owner;
    address public alice;

    DKIMRegistry dkimRegistry;
    IProofOfTwitter_Groth16Verifier groth16Verifier;
    ZKEmailProof zkEmailProof;
    ProofOfTwitter_Verifier verifier;

    Proof proof;
    uint256[8] publicOutputs;
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
        vm.rollFork(21546387);

        owner = address(1);
        // We're setting alice to a value in the publicOutputs array, but this is a bit of a hack as
        // the value is not actually an owner address according to the original proof
        alice = address(8194671501497130006289079365482);

        dkimRegistry = new DKIMRegistry(owner);
        groth16Verifier = IProofOfTwitter_Groth16Verifier(DEPLOYED_VERIFIER);
        zkEmailProof = new ZKEmailProof(owner);
        verifier = new ProofOfTwitter_Verifier(
            address(dkimRegistry),
            address(groth16Verifier),
            address(zkEmailProof)
        );

        proof = Proof({
            a: [
                19014357250828634823182307903338300175090724504730053381041748437315271534735,
                4696736584263604331558996548418964379075298974995081308895901094664013414220
            ],
            b: [
                [
                    2166854163583550883937146753361462030743291876364316237947986608198907783650,
                    14921817661383102514061633663737655278513086830555952892732430740252347498809
                ],
                [
                    11235379133743098261599914832460495433729372576618727742968307957676590058492,
                    3006462367602337020834301360939705755022854337806187866519397082844272003175
                ]
            ],
            c: [
                12074728462639618962329481847660153022131782339938453563695793425878404386242,
                6191198095661398861156257556629402133673552135534202763283703513866655390104
            ]
        });
        publicOutputs = [
            1983664618407009423875829639306275185491946247764487749439145140682408188330,
            8194671501497130006289079365482,
            0,
            0,
            0,
            116992936385065960565912052140412177013034054326367951776368268607948945456,
            81467355455428621312079923,
            0
        ];
        publicOutputFieldNames = ["handle"];
        to = alice;
        blueprintId = 1;
        toAddressIndex = 1;

        domainName = "x.com";
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
            memory expectedDecodedPublicOutputs = '{"handle":"john_guilding"}';

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
        assertEq(metadata.publicOutputs[5], publicOutputs[5]);
        assertEq(metadata.publicOutputs[6], publicOutputs[6]);
        assertEq(metadata.publicOutputs[7], publicOutputs[7]);
        assertEq(metadata.decodedPublicOutputs, expectedDecodedPublicOutputs);
    }
}
