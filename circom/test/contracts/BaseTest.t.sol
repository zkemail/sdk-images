// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {DKIMRegistry} from "@zk-email/contracts/DKIMRegistry.sol";
import {ZKEmailProof, Proof} from "../../contracts/ZKEmailProof.sol";
import {MockGroth16Verifier} from "../../contracts/test/MockGroth16Verifier.sol";

import {TestVerifierHarness} from "./TestVerifierHarness.sol";

contract BaseTest is Test {
    DKIMRegistry dkimRegistry;
    MockGroth16Verifier groth16Verifier;
    ZKEmailProof zkEmailProof;
    TestVerifierHarness verifier;

    address public owner;
    address public alice;
    address public bob;

    uint256 blueprintId;
    Proof proof;
    uint256[] publicOutputs = new uint256[](16);
    string[3] publicOutputFieldNames;
    string decodedPublicOutputs;
    address to;
    uint256 toAddressStartIndex;

    string domainName;
    bytes32 publicKeyHash;

    function setUp() public virtual {
        owner = vm.addr(1);
        alice = vm.addr(2);
        bob = vm.addr(3);

        dkimRegistry = new DKIMRegistry(owner);
        groth16Verifier = new MockGroth16Verifier();
        zkEmailProof = new ZKEmailProof(owner);

        verifier = new TestVerifierHarness(
            address(dkimRegistry),
            address(groth16Verifier),
            address(zkEmailProof)
        );

        blueprintId = 1;
        proof = Proof({
            a: [uint256(1), uint256(2)],
            b: [[uint256(3), uint256(4)], [uint256(5), uint256(6)]],
            c: [uint256(7), uint256(8)]
        });
        publicOutputs[
            0
        ] = 8011766048918436304234337347171138895102985966651471271518887910697337713809;
        publicOutputs[1] = 1852337994;
        publicOutputs[2] = 0;
        publicOutputs[3] = 0;
        publicOutputs[
            4
        ] = 57377328031630107749936499991080080299453129889983133523939340367986255164;
        publicOutputs[5] = 5438187003054578043726741043588292439992695;
        publicOutputs[6] = 0;
        publicOutputs[7] = 0;
        publicOutputs[8] = 0;
        publicOutputs[9] = 0;
        publicOutputs[10] = 0;
        publicOutputs[
            11
        ] = 203411379827238570491176173578436868093537450257881964775793530671143609719;
        publicOutputs[12] = 2037169922858342507125;
        publicOutputs[13] = 0;
        publicOutputs[
            14
        ] = 93982438239657877452062018043752344615292808895017758601572026041563772976;
        publicOutputs[15] = 123778951240410959589749349;
        publicOutputFieldNames = [
            "recipient_name",
            "proposal_title",
            "rejection_line"
        ];
        decodedPublicOutputs = '{"recipient_name":"John","proposal_title":"<em>Making Smart Accounts easy with ZK Email</em>","rejection_line":"we were unable to accept this submission"}';
        to = alice;
        blueprintId = 1;
        toAddressStartIndex = 14;

        domainName = "domain.org";
        publicKeyHash = bytes32(publicOutputs[0]);

        vm.startPrank(owner);
        zkEmailProof.addVerifier(address(verifier));
        dkimRegistry.setDKIMPublicKeyHash(domainName, publicKeyHash);
        vm.stopPrank();
    }
}
