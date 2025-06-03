// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {Proof} from "../../../contracts/ZKEmailProof.sol";
import {NFTSVG} from "../../../contracts/NFTSVG.sol";
import {BaseTest} from "../BaseTest.t.sol";

contract ZKEmailProof_TokenURI_Test is BaseTest {
    using Strings for uint256;
    using Strings for address;

    function setUp() public override {
        super.setUp();
    }

    function test_ZKEmailProof_TokenURI() public {
        vm.prank(address(verifier));
        zkEmailProof.mintProof(
            alice,
            blueprintId,
            proof,
            publicOutputs,
            decodedPublicOutputs
        );

        uint256 tokenId = 0;
        string memory tokenUri = zkEmailProof.tokenURI(tokenId);
        NFTSVG.SVGParams memory svgParams = NFTSVG.generateSVGParams(
            decodedPublicOutputs,
            tokenId
        );
        string memory svg = NFTSVG.generateSVG(svgParams);

        string memory expectedJson = string.concat(
            '{"name":"ZKEmail Proof #0",',
            '"description":"Soulbound NFT representing a valid ZK Email proof for an account",',
            '"image":"data:image/svg+xml;base64,',
            Base64.encode(bytes(svg)),
            '","attributes":[',
            '{"trait_type":"Blueprint ID","value":1},',
            '{"trait_type":"Proof","value":[[1,2],[[3,4],[5,6]],[7,8]]},',
            '{"trait_type":"Public Outputs","value":[',
            "8011766048918436304234337347171138895102985966651471271518887910697337713809,",
            "1852337994,",
            "0,",
            "0,",
            "57377328031630107749936499991080080299453129889983133523939340367986255164,",
            "5438187003054578043726741043588292439992695,",
            "0,",
            "0,",
            "0,",
            "0,",
            "0,",
            "203411379827238570491176173578436868093537450257881964775793530671143609719,",
            "2037169922858342507125,",
            "0,",
            "93982438239657877452062018043752344615292808895017758601572026041563772976,",
            "123778951240410959589749349",
            "]},",
            '{"trait_type":"Decoded Public Outputs","value":',
            '{"recipient_name":"John","proposal_title":"<em>Making Smart Accounts easy with ZK Email</em>","rejection_line":"we were unable to accept this submission"}},'
            '{"trait_type":"Verifier","value":"',
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
