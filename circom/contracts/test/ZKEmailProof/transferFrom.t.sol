// // SPDX-License-Identifier: MIT
// pragma solidity ^0.8.13;

// import "forge-std/Test.sol";
// import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
// import {ZKEmailProof, Proof, ZKEmailProofMetadata} from "../../src/ZKEmailProof.sol";
// import {TestVerifier} from "../../src/test/TestVerifier.sol";

// contract ZKEmailProof_SafeTransferFrom_Test is Test {
//     ZKEmailProof public zkEmailProof;
//     address public alice;
//     address public bob;
//     address public admin;

//     uint256 blueprintId;
//     Proof proof;
//     uint256[] publicOutputs = new uint256[](1);
//     string decodedPublicOutputs;
//     TestVerifier verifier;

//     function setUp() public {
//         alice = address(1);
//         bob = address(2);
//         admin = address(3);
//         zkEmailProof = new ZKEmailProof(admin);

//         blueprintId = 1;
//         proof = Proof({
//             a: [uint256(1), uint256(2)],
//             b: [[uint256(3), uint256(4)], [uint256(5), uint256(6)]],
//             c: [uint256(7), uint256(8)]
//         });
//         publicOutputs[0] = uint256(uint160(alice));
//         decodedPublicOutputs = '"to": 1, "username": "John Smith"';
//         verifier = new TestVerifier(address(zkEmailProof));

//         vm.prank(admin);
//         zkEmailProof.setVerifier(address(verifier));
//         vm.stopPrank();
//     }

//     function test_ZKEmailProof_SafeTransferFrom_RevertWhen_TransferToBob()
//         public
//     {
//         vm.prank(address(verifier));
//         zkEmailProof.safeMint(
//             alice,
//             blueprintId,
//             proof,
//             publicOutputs,
//             decodedPublicOutputs
//         );
//         uint256 tokenId = 0;

//         vm.expectRevert(ZKEmailProof.CannotTransferSoulboundToken.selector);
//         zkEmailProof.safeTransferFrom(alice, bob, tokenId);
//     }
// }