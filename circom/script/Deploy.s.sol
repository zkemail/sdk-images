// // SPDX-License-Identifier: UNLICENSED
// pragma solidity ^0.8.13;

// TODO: Update this script
// import "forge-std/Script.sol";
// import "@zk-email/contracts/interfaces/IDKIMRegistry.sol";
// import "@zk-email/contracts/DKIMRegistry.sol";
// import {IGroth16Verifier} from "../contracts/IGroth16Verifier.sol";
// // import "./tmp/verifier.sol";
// // import "./tmp/contract.sol";

// contract Deploy is Script {
//     IDKIMRegistry dkimRegistry;

//     function run() external {
//         uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
//         if (deployerPrivateKey == 0) {
//             console.log("PRIVATE_KEY env var not set");
//             return;
//         }

//         vm.startBroadcast(deployerPrivateKey);
//         address dkimRegistryAddr;
//         try vm.envAddress("DKIM_REGISTRY") returns (address addr) {
//             dkimRegistryAddr = addr;
//         } catch {
//             dkimRegistryAddr = address(0);
//         }

//         if (dkimRegistryAddr == address(0)) {
//             dkimRegistry = new DKIMRegistry(msg.sender);
//             dkimRegistryAddr = address(dkimRegistry);
//         }

//         dkimRegistry = IDKIMRegistry(dkimRegistryAddr);

//         Groth16Verifier verifier = new Groth16Verifier();
//         Contract circuitContract = new Contract(dkimRegistry, verifier);
//         vm.stopBroadcast();

//         console.log("Deployed Groth16Verifier at", address(verifier));
//         console.log("Deployed Contract at", address(circuitContract));
//         console.log("Deployed DKIMRegistry at", dkimRegistryAddr);
//         return;
//     }
// }
