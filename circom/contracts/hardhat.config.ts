import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-ethers";
import "@nomicfoundation/hardhat-ignition-ethers";
import "@nomicfoundation/hardhat-verify";
import "@parity/hardhat-polkadot";
import "dotenv/config";

const config: HardhatUserConfig = {
  networks: {
    hardhat: {
      polkadot: true,
      nodeConfig: {
        nodeBinaryPath: "./bin/dev-node",
        rpcPort: 8000,
        dev: true,
      },
      adapterConfig: {
        adapterBinaryPath: "./bin/eth-rpc",
        dev: true,
      },
    },
    localNode: {
      polkadot: true,
      url: `http://127.0.0.1:8545`,
    },
    polkadotHubTestnet: {
      polkadot: true,
      url: "https://services.polkadothub-rpc.com/testnet",
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
    },
  },
  solidity: {
    version: "0.8.30",
    settings: {
      optimizer: {
        enabled: true,
        runs: 10000,
      },
      evmVersion: "prague",
    },
  },
  resolc: {
    version: "0.5.0",
    settings: {
      optimizer: {
        enabled: true,
        runs: 10000,
      },
    },
  },
  paths: {
    sources: "src",
    tests: "hh-tests",
    cache: "hh-cache",
    artifacts: "hh-artifacts",
    ignition: "hh-ignition",
  },
};

export default config;
