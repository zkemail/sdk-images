// SPDX-License-Identifier: MIT
pragma solidity ^0.8.30;

/**
 * @title TestBlueprintFixture
 * @notice A real Groth16 proof + 7-element public-signal array for a real test blueprint (sender
 * domain x.com), obtained via real remote proving for a real x.com email (DKIM d=x.com, dkim=pass
 * at delivery). See README.md in this directory for full provenance. pB here has the G2-coordinate
 * swap `createCallData()` (zk-email-sdk-js's own on-chain submission helper) applies, matching what
 * `abi.decode(proof, (uint256[2], uint256[2][2], uint256[2]))` expects.
 */
library TestBlueprintFixture {
    function pA() internal pure returns (uint256[2] memory) {
        return [
        0x62dfeb33a9766733596cb3afd93efe6dc7b4da8c8fb2273a937c44db7a5fd01,
        0x1f04f977f0ec63518af809707011b392ed821c6d2ac1b6efe8102dc7491eeca2
        ];
    }

    function pB() internal pure returns (uint256[2][2] memory) {
        return [
            [
                0x14db43a9062f8a63b583849abf3d463e1c836b6cd204b3ef8fb8e8bf1e937885,
                0x7507a572ef961d5ceb69d2cbce8934dd0f6c259e6438fd5b714f523ce814ad4
            ],
            [
                0x3c96dda17f00004ab518ca9932940f60a928baeae6e4be930a5ad047dc93bb1,
                0x1a54851d5eadd3b7d7032c7e92dced2bfecd480308d8ab4827ab43065cc94795
            ]
        ];
    }

    function pC() internal pure returns (uint256[2] memory) {
        return [
        0x2a1116499bebaa1b5e414ee0bd74239452c0a74ad7a015a44b637669ca5819dc,
        0x1437b7f283e89d8d2cecc7d6f4f40553f70b25f3a32bcbca2fbb34f793c75cf9
        ];
    }

    function publicInputs() internal pure returns (bytes32[] memory result) {
        result = new bytes32[](7);
        result[0] = bytes32(uint256(0x462b6e208f3552371d7c7d2fbeb31691e5f789b9e5f0bdfaa68a6a84f01d9aa));
        result[1] = bytes32(uint256(0x85fb869a94511ccbaaf108f91f59b407));
        result[2] = bytes32(uint256(0xf36f89025341ed6536cbe2d0d338b7a1));
        result[3] = bytes32(uint256(0x6d6f632e78406f666e69));
        result[4] = bytes32(uint256(0x0));
        result[5] = bytes32(uint256(0x0));
        result[6] = bytes32(uint256(0x0));
    }
}
