// SPDX-License-Identifier: GPL-2.0-or-later
pragma solidity >=0.7.6;

import {Base64} from "@openzeppelin/contracts/utils/Base64.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {StringUtils} from "@zk-email/contracts/utils/StringUtils.sol";

/// @title NFTSVG
/// @notice Provides a function for generating an SVG associated with a ZK Email Proof
library NFTSVG {
    using Strings for uint256;
    using Strings for address;

    struct SVGParams {
        // TODO: make this generic
        string decodedPublicOutputs;
        uint256 tokenId;
        string color0;
        string color1;
        string color2;
        string color3;
        string x1;
        string y1;
        string x2;
        string y2;
        string x3;
        string y3;
    }

    // TODO: make this generic
    function generateSVG(
        SVGParams memory params
    ) internal pure returns (string memory svg) {
        return
            string(
                abi.encodePacked(
                    generateSVGDefs(params),
                    generateSVGBorderText(params.decodedPublicOutputs),
                    generateSVGCardMantle(params.decodedPublicOutputs),
                    generateSVGLogo(),
                    "</svg>"
                )
            );
    }

    function generateSVGDefs(
        SVGParams memory params
    ) private pure returns (string memory svg) {
        svg = string(
            abi.encodePacked(
                '<svg width="290" height="500" viewBox="0 0 290 500" xmlns="http://www.w3.org/2000/svg"',
                " xmlns:xlink='http://www.w3.org/1999/xlink'>",
                "<defs>",
                '<filter id="f1"><feImage result="p0" xlink:href="data:image/svg+xml;base64,',
                Base64.encode(
                    bytes(
                        abi.encodePacked(
                            "<svg width='290' height='500' viewBox='0 0 290 500' xmlns='http://www.w3.org/2000/svg'><rect width='290px' height='500px' fill='#",
                            params.color0,
                            "'/></svg>"
                        )
                    )
                ),
                '"/><feImage result="p1" xlink:href="data:image/svg+xml;base64,',
                Base64.encode(
                    bytes(
                        abi.encodePacked(
                            "<svg width='290' height='500' viewBox='0 0 290 500' xmlns='http://www.w3.org/2000/svg'><circle cx='",
                            params.x1,
                            "' cy='",
                            params.y1,
                            "' r='120px' fill='#",
                            params.color1,
                            "'/></svg>"
                        )
                    )
                ),
                '"/><feImage result="p2" xlink:href="data:image/svg+xml;base64,',
                Base64.encode(
                    bytes(
                        abi.encodePacked(
                            "<svg width='290' height='500' viewBox='0 0 290 500' xmlns='http://www.w3.org/2000/svg'><circle cx='",
                            params.x2,
                            "' cy='",
                            params.y2,
                            "' r='120px' fill='#",
                            params.color2,
                            "'/></svg>"
                        )
                    )
                ),
                '" />',
                '<feImage result="p3" xlink:href="data:image/svg+xml;base64,',
                Base64.encode(
                    bytes(
                        abi.encodePacked(
                            "<svg width='290' height='500' viewBox='0 0 290 500' xmlns='http://www.w3.org/2000/svg'><circle cx='",
                            params.x3,
                            "' cy='",
                            params.y3,
                            "' r='100px' fill='#",
                            params.color3,
                            "'/></svg>"
                        )
                    )
                ),
                '" /><feBlend mode="overlay" in="p0" in2="p1" /><feBlend mode="exclusion" in2="p2" /><feBlend mode="overlay" in2="p3" result="blendOut" /><feGaussianBlur ',
                'in="blendOut" stdDeviation="42" /></filter> <clipPath id="corners"><rect width="290" height="500" rx="42" ry="42" /></clipPath>',
                '<path id="text-path-a" d="M40 12 H250 A28 28 0 0 1 278 40 V460 A28 28 0 0 1 250 488 H40 A28 28 0 0 1 12 460 V40 A28 28 0 0 1 40 12 z" />',
                '<path id="minimap" d="M234 444C234 457.949 242.21 463 253 463" />',
                '<filter id="top-region-blur"><feGaussianBlur in="SourceGraphic" stdDeviation="24" /></filter>',
                '<linearGradient id="grad-up" x1="1" x2="0" y1="1" y2="0"><stop offset="0.0" stop-color="white" stop-opacity="1" />',
                '<stop offset=".9" stop-color="white" stop-opacity="0" /></linearGradient>',
                '<linearGradient id="grad-down" x1="0" x2="1" y1="0" y2="1"><stop offset="0.0" stop-color="white" stop-opacity="1" /><stop offset="0.9" stop-color="white" stop-opacity="0" /></linearGradient>',
                '<mask id="fade-up" maskContentUnits="objectBoundingBox"><rect width="1" height="1" fill="url(#grad-up)" /></mask>',
                '<mask id="fade-down" maskContentUnits="objectBoundingBox"><rect width="1" height="1" fill="url(#grad-down)" /></mask>',
                '<mask id="none" maskContentUnits="objectBoundingBox"><rect width="1" height="1" fill="white" /></mask>',
                '<linearGradient id="grad-symbol"><stop offset="0.7" stop-color="white" stop-opacity="1" /><stop offset=".95" stop-color="white" stop-opacity="0" /></linearGradient>',
                '<mask id="fade-symbol" maskContentUnits="userSpaceOnUse"><rect width="290px" height="300px" fill="url(#grad-symbol)" /></mask></defs>',
                '<g clip-path="url(#corners)">',
                '<rect fill="',
                params.color0,
                '" x="0px" y="0px" width="290px" height="500px" />',
                '<rect style="filter: url(#f1)" x="0px" y="0px" width="290px" height="500px" />',
                ' <g style="filter:url(#top-region-blur); transform:scale(1.5); transform-origin:center top;">',
                '<rect fill="none" x="0px" y="0px" width="290px" height="500px" />',
                '<ellipse cx="50%" cy="0px" rx="180px" ry="120px" fill="#041C32" opacity="0.85" /></g>',
                '<rect x="0" y="0" width="290" height="500" rx="42" ry="42" fill="rgba(0,0,0,0)" stroke="rgba(255,255,255,0.2)" /></g>'
            )
        );
    }

    // TODO: make this generic
    function generateSVGBorderText(
        string memory decodedPublicOutputs
    ) private pure returns (string memory svg) {
        svg = string(
            abi.encodePacked(
                '<text text-rendering="optimizeSpeed">',
                '<textPath startOffset="-100%" fill="white" font-family="\'Courier New\', monospace" font-size="10px" xlink:href="#text-path-a">',
                decodedPublicOutputs,
                ' <animate additive="sum" attributeName="startOffset" from="0%" to="100%" begin="0s" dur="30s" repeatCount="indefinite" />',
                '</textPath> <textPath startOffset="0%" fill="white" font-family="\'Courier New\', monospace" font-size="10px" xlink:href="#text-path-a">',
                ' <animate additive="sum" attributeName="startOffset" from="0%" to="100%" begin="0s" dur="30s" repeatCount="indefinite" /> </textPath></text>'
            )
        );
    }

    function generateOrdinalSuffix(
        uint256 degree
    ) private pure returns (string memory ordinal) {
        uint256 degree10 = degree % 10;
        uint256 degree100 = degree % 100;
        if (degree10 == 1 && degree100 != 11) {
            ordinal = string("st");
        } else if (degree10 == 2 && degree100 != 12) {
            ordinal = string("nd");
        } else if (degree10 == 3 && degree100 != 13) {
            ordinal = string("rd");
        } else {
            ordinal = string("th");
        }
    }

    // TODO: make this generic
    function generateSVGCardMantle(
        string memory decodedPublicOutputs
    ) private pure returns (string memory svg) {
        svg = string(
            abi.encodePacked(
                '<g mask="url(#fade-symbol)"><rect fill="none" x="0px" y="0px" width="290px" height="400px" /> <text y="70px" x="32px" fill="white" font-family="\'Courier New\', monospace" font-weight="200" font-size="36px">',
                "ZK Email Proof",
                '</text><text y="115px" x="32px" fill="white" font-family="\'Courier New\', monospace" font-weight="200" font-size="36px">',
                decodedPublicOutputs,
                "</text></g>",
                '<rect x="16" y="16" width="258" height="468" rx="26" ry="26" fill="rgba(0,0,0,0)" stroke="rgba(255,255,255,0.2)" />'
            )
        );
    }

    function generateSVGLogo() private pure returns (string memory svg) {
        svg = string(
            abi.encodePacked(
                '<g style="transform:translate(84px, 300px) scale(5)"><g><path d="M4.40434 13.6099C3.51517 13.1448 3 12.5924 3 12C3 10.3431 7.02944 9 12 9C16.9706 9 21 10.3431 21 12C21 12.7144 20.2508 13.3705 19 13.8858" stroke="white"  stroke-linecap="round" stroke-linejoin="round" fill="none"/>',
                '<path d="M12 11.01L12.01 10.9989" stroke="white" stroke-linecap="round" stroke-linejoin="round" fill="none"/>',
                '<path d="M16.8827 6C16.878 4.97702 16.6199 4.25309 16.0856 3.98084C14.6093 3.22864 11.5832 6.20912 9.32664 10.6379C7.07005 15.0667 6.43747 19.2668 7.91374 20.019C8.44117 20.2877 9.16642 20.08 9.98372 19.5" stroke="white"  stroke-linecap="round" stroke-linejoin="round" fill="none"/>',
                '<path d="M9.60092 4.25164C8.94056 3.86579 8.35719 3.75489 7.91369 3.98086C6.43742 4.73306 7.06999 8.93309 9.32658 13.3619C11.5832 17.7907 14.6092 20.7712 16.0855 20.019C17.3977 19.3504 17.0438 15.9577 15.3641 12.1016" stroke="white"  stroke-linecap="round" stroke-linejoin="round" fill="none"/>',
                '<animateTransform attributeName="transform" type="rotate" from="0 12 12" to="360 12 12" dur="10s" repeatCount="indefinite"/></g></g>'
            )
        );
    }

    // TODO: make this generic
    function generateSVGParams(
        string memory decodedPublicOutputs,
        uint256 tokenId
    ) internal pure returns (NFTSVG.SVGParams memory) {
        return
            NFTSVG.SVGParams({
                decodedPublicOutputs: decodedPublicOutputs,
                tokenId: tokenId,
                color0: tokenToColorHex(
                    uint256(bytes32(bytes(decodedPublicOutputs))),
                    136
                ),
                color1: tokenToColorHex(
                    uint256(bytes32(bytes(decodedPublicOutputs))),
                    136
                ),
                color2: tokenToColorHex(
                    uint256(bytes32(bytes(decodedPublicOutputs))),
                    0
                ),
                color3: tokenToColorHex(
                    uint256(bytes32(bytes(decodedPublicOutputs))),
                    0
                ),
                x1: scale(
                    getCircleCoord(
                        uint256(bytes32(bytes(decodedPublicOutputs))),
                        16,
                        tokenId
                    ),
                    0,
                    255,
                    16,
                    274
                ),
                y1: scale(
                    getCircleCoord(
                        uint256(bytes32(bytes(decodedPublicOutputs))),
                        16,
                        tokenId
                    ),
                    0,
                    255,
                    100,
                    484
                ),
                x2: scale(
                    getCircleCoord(
                        uint256(bytes32(bytes(decodedPublicOutputs))),
                        32,
                        tokenId
                    ),
                    0,
                    255,
                    16,
                    274
                ),
                y2: scale(
                    getCircleCoord(
                        uint256(bytes32(bytes(decodedPublicOutputs))),
                        32,
                        tokenId
                    ),
                    0,
                    255,
                    100,
                    484
                ),
                x3: scale(
                    getCircleCoord(
                        uint256(bytes32(bytes(decodedPublicOutputs))),
                        48,
                        tokenId
                    ),
                    0,
                    255,
                    16,
                    274
                ),
                y3: scale(
                    getCircleCoord(
                        uint256(bytes32(bytes(decodedPublicOutputs))),
                        48,
                        tokenId
                    ),
                    0,
                    255,
                    100,
                    484
                )
            });
    }

    function tokenToColorHex(
        uint256 token,
        uint256 offset
    ) internal pure returns (string memory str) {
        return string(StringUtils.toHexStringNoPrefix(token >> offset, 3));
    }

    function scale(
        uint256 n,
        uint256 inMn,
        uint256 inMx,
        uint256 outMn,
        uint256 outMx
    ) private pure returns (string memory) {
        return
            Strings.toString(
                ((n - inMn) * (outMx - outMn)) / (inMx - inMn) + (outMn)
            );
    }

    // TODO: make this generic
    function getCircleCoord(
        uint256 param,
        uint256 offset,
        uint256 tokenId
    ) internal pure returns (uint256) {
        return (sliceTokenHex(param, offset) * tokenId) % 255;
    }

    // TODO: make this generic
    function sliceTokenHex(
        uint256 param,
        uint256 offset
    ) internal pure returns (uint256) {
        return uint256(uint8(param >> offset));
    }
}
