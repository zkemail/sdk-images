import { HardhatUserConfig } from "hardhat/config";
import "@nomicfoundation/hardhat-ethers";
import "@nomicfoundation/hardhat-ignition-ethers";
import "@nomicfoundation/hardhat-verify";
import "@parity/hardhat-polkadot";
import "dotenv/config";

const rpcUrl = process.env.RPC_URL;
const accounts = process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [];

const config: HardhatUserConfig = {
  networks: {
    hardhat: {
      polkadot: {
        target: "pvm",
      },
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
    localEvm: {
      url: "http://127.0.0.1:8545/",
      ...(accounts.length ? { accounts } : {}),
    },
    localPvm: {
      polkadot: {
        target: "pvm",
      },
      url: `http://127.0.0.1:8545`,
      ...(accounts.length ? { accounts } : {}),
    },
    // Polkadot Hub Testnet
    "420420417": {
      polkadot: {
        target: "pvm",
      },
      url: rpcUrl || "https://services.polkadothub-rpc.com/testnet",
      accounts,
    },
    // Base Sepolia
    "84532": {
      url: rpcUrl || "https://sepolia.base.org",
      accounts,
    },
    // Ethereum Sepolia
    "11155111": {
      url: rpcUrl || "https://ethereum-sepolia-rpc.publicnode.com",
      accounts,
    },
  },
  etherscan: {
    apiKey: process.env.ETHERSCAN_API_KEY,
    customChains: [
      {
        chainId: 84532,
        network: "84532",
        urls: {
          apiURL: "https://api-sepolia.basescan.org/api",
          browserURL: "https://sepolia.basescan.org/",
        },
      },
      {
        chainId: 11155111,
        network: "11155111",
        urls: {
          apiURL: "https://api-sepolia.etherscan.io/api",
          browserURL: "https://sepolia.etherscan.io/",
        },
      },
    ],
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
  ignition: {
    // IGN401 fix for PolkaVM / Paseo AssetHub:
    // The EVM-RPC adapter briefly returns null from eth_getTransaction right after
    // a tx is included (indexing gap). Default maxRetries=10 × retryInterval=1s
    // gives only 10s to detect the tx before IGN401 is thrown. 60s is enough.
    maxRetries: 60,
    // Stop Ignition from sending replacement txs (same nonce, higher gas): the
    // EVM-RPC adapter does not handle them well and can make the original tx also
    // appear dropped.
    disableFeeBumping: true,
    // PolkaVM uses GRANDPA deterministic finality — 1 confirmation is truly final.
    // Default is 5.
    requiredConfirmations: 1,
  },
};

export default config;
