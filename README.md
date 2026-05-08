# SDK Images

This project automates the process of generating and deploying a zk-SNARK circuit for email validation. It takes a blueprint configuration file that specifies regex patterns and circuit parameters, generates Circom circuits for each regex pattern, and combines them with a DKIM verification circuit. The project then compiles the circuit, generates proving and verification keys, and deploys smart contracts for on-chain proof verification and DKIM registry integration.

## Key Features

- Automates the generation of Circom circuits for email validation based on regex patterns.
- Determines optimal power-of-tau parameters based on constraint count.
- Compiles the circuit using the `circom` compiler and generates a witness calculator.
- Generates proving and verification keys using the Groth16 ZK proof system.
- Creates a Solidity verifier contract for on-chain ZK proof verification.
- Deploys smart contracts for DKIM registry integration and proof validation.

## How it works

It consists of several steps:

1. **Configuration and Setup**:
   - Loads a configuration payload and establishes a database connection.
   - Sets up a temporary directory (`tmp`) and removes any existing contents.

2. **Circuit Generation**:
   - Generates regex circuits based on the provided blueprint.
   - Generates a main circuit template using the blueprint and writes it to a file.

3. **Circuit Compilation**:
   - Compiles the circuit using the `circom` compiler.
   - Determines the required power of tau (ptau) based on the number of constraints and wires.
   - Compiles the circuit binary using a witness generator.

4. **Key Generation**:
   - Downloads the ptau file and generates a zkey.
   - Contributes to the zkey and exports the verification key.

5. **Smart Contract Deployment**:
   - Prepares the contract data and creates a contract.
   - Generates and deploys a verifier contract.

6. **Cleanup and File Upload**:
   - Cleans up the temporary files and compresses the zkey chunks.
   - Zips various files (circuit, circuit_cpp, zkey, verification key, witness calculator, etc.).
   - Uploads the generated files to specified upload URLs.

7. **Database Update**:
   - Updates the verifier contract address in the database.

## Directory Structure

```
.
├── Cargo.toml          # Workspace configuration file for the Rust project.
├── LICENSE             # License file for the project.
├── README.md           # Main README file providing an overview of the project.
│
├── circom
│   ├── Cargo.toml      # Cargo configuration file for the Circom project.
│   ├── src             # Main Rust code for compiling and deploying circuits.
│   └── templates       # Tera templates for generating Circom and Solidity code.
│
├── noir                  # Noir-based pipeline (generation, compile, run)
│   ├── Cargo.toml
│   ├── src
│   └── templates         # Tera templates for Noir
│
└── sdk-utils             # Shared utilities for both pipelines
    ├── Cargo.toml
    └── src
```

## Prerequisites (macOS)

- Rust toolchain (rustup) and Cargo
- curl, zip, gzip
- Node.js and npm (v18+ recommended)

Install the external ZK/crypto toolchain:

1) Noir (via noirup)

```bash
curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash
# Start a new shell or source your profile if prompted, e.g.:
source ~/.zshrc

# Install a specific Nargo/Noir version (recommended for this repo)
noirup --version 1.0.0-beta.5
```

2) Barretenberg CLI (bbup)

```bash
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
# Start a new shell or source your profile if prompted
bbup --version 0.84.0
```

3) Foundry (Solidity toolchain)

```bash
curl -L https://foundry.paradigm.xyz | bash
# After install:
source ~/.zshenv
foundryup
```

4) snarkjs and yarn (for Circom path)

```bash
npm install -g yarn snarkjs
```

Verify installations are on PATH:

```bash
which nargo && nargo --version
which noirup
which bb && bb --version
which bbup
which forge && forge --version
which snarkjs && snarkjs --version
which yarn && yarn --version
```

## Building and Running

To build and run the application, follow these steps:

1. Install the necessary dependencies, including Rust and required external tools (`circom`, `snarkjs`, `bazel`, `curl`, `gzip`, `zip`).
2. Clone the repository and navigate to the project directory.
3. Build the project using `cargo build`.
4. Run the application using `cargo run`.

## License

This project is licensed under the [MIT License](LICENSE).
