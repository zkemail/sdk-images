// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import {IVerifier} from "../../../contracts/IVerifier.sol";
import {Proof} from "../../../contracts/ZKEmailProof.sol";
import {BaseTest} from "../ZKEmailProof/BaseTest.t.sol";

contract ZKEmailProof_Fork_Test is BaseTest {
    address deployedVerifier = 0x7019c2E274c77dd6E9e4C2707068BC6e690eA0AF;

    function setUp() public override {
        super.setUp();
        string memory BASE_SEPOLIA_RPC_URL = vm.envString(
            "BASE_SEPOLIA_RPC_URL"
        );
        vm.createSelectFork(BASE_SEPOLIA_RPC_URL);
        vm.rollFork(20880810);
    }

    function testVerify() public view {
        Proof memory proof = Proof({
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

        uint256[5] memory publicOutputs = [
            3024598485745563149860456768272954250618223591034926533254923041921841324429,
            2440484440003696966756646629102736908273017697,
            0,
            0,
            0
        ];

        IVerifier(deployedVerifier).verify(
            proof.a,
            proof.b,
            proof.c,
            publicOutputs
        );
    }
}
