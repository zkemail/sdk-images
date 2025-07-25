# ZK Email SDK Images

This repository contains the ZK Email SDK for building and compiling zero-knowledge circuits for email verification.

## Prerequisites

Before running this project locally, you need to install several dependencies that are required for zero-knowledge circuit compilation and blockchain interactions.

## Local Setup Instructions

### 1. System Dependencies

First, install the required system packages:

```bash
# Update package list
sudo apt update

# Install essential build tools and libraries
sudo apt install -y \
    build-essential \
    libgmp-dev \
    libomp-dev \
    nasm \
    nlohmann-json3-dev \
    wget \
    git \
    curl \
    ca-certificates \
    libssl-dev \
    pkg-config
```

### 2. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### 3. Install Node.js and Yarn

```bash
# Install Node.js (version 18 or later recommended)
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install Yarn globally
npm install -g yarn
```

### 4. Install Bazelisk

Bazelisk is required for building Tachyon dependencies:

```bash
wget https://github.com/bazelbuild/bazelisk/releases/latest/download/bazelisk-linux-amd64
chmod +x bazelisk-linux-amd64
sudo mv bazelisk-linux-amd64 /usr/local/bin/bazel
```

### 5. Set up Tachyon

Tachyon is required for the circom witness generator:

```bash
# Clone Tachyon repository
cd ~
git clone https://github.com/kroma-network/tachyon.git
cd tachyon

# Build circom dependencies with Bazel
cd vendors/circom
CARGO_BAZEL_REPIN=true bazel build --config opt //circomlib/build:compile_witness_generator

# Set environment variable
export TACHYON_DIR=~/tachyon
echo 'export TACHYON_DIR=~/tachyon' >> ~/.bashrc
```

### 6. Install Circom (Correct Version)

**Important**: This project requires circom v2.1.9. If you have an older version installed, remove it first:

```bash
# Remove old circom installation (if installed via npm)
sudo npm uninstall -g circom

# Or if installed via apt
sudo apt remove circom

# Install circom v2.1.9 from source
git clone https://github.com/iden3/circom.git
cd circom
git checkout tags/v2.1.9 -b v2.1.9
cargo build --release
cargo install --path circom

# Clear bash command cache
hash -r

# Verify installation
circom --version  # Should show 2.1.9
```

### 7. Install Foundry

Foundry is required for Solidity contract compilation:

```bash
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash

# Add to PATH
export PATH="$HOME/.foundry/bin:$PATH"
echo 'export PATH="$HOME/.foundry/bin:$PATH"' >> ~/.bashrc

# Install latest version
foundryup

# Verify installation
forge --version
```

### 8. Set Node.js Memory Limits

For large circuit compilation:

```bash
export NODE_OPTIONS=--max_old_space_size=65536
echo 'export NODE_OPTIONS=--max_old_space_size=65536' >> ~/.bashrc
```

### 9. Environment Variables

Make sure all required environment variables are set:

```bash
# Reload your shell configuration
source ~/.bashrc

# Verify environment variables
echo $TACHYON_DIR
echo $NODE_OPTIONS
echo $PATH
```

## Running the Project

### Build and Run

```bash
# Navigate to the circom directory
cd circom

# Install Node.js dependencies
yarn install

# Build and run the project
cargo run
```

### Docker Alternative

If you prefer to use Docker (which has all dependencies pre-installed):

```bash
# Build the Docker image
docker build -f circom/Dockerfile -t zkemail-sdk .

# Run the container
docker run zkemail-sdk
```

## Troubleshooting

### Common Issues

1. **"circom: command not found"** or wrong version
   - Make sure you installed circom v2.1.9 from source
   - Run `hash -r` to clear bash command cache
   - Verify `~/.cargo/bin` is in your PATH

2. **"forge: not found"**
   - Install Foundry as described above
   - Make sure `~/.foundry/bin` is in your PATH

3. **Bazel build failures**
   - Ensure you have all system dependencies installed
   - Try running the Bazel build command again

4. **Memory issues during compilation**
   - Increase Node.js memory limit: `export NODE_OPTIONS=--max_old_space_size=65536`

5. **Missing TACHYON_DIR**
   - Verify the environment variable is set: `echo $TACHYON_DIR`
   - Should point to your tachyon directory (e.g., `~/tachyon`)

### Verification Commands

Run these commands to verify your setup:

```bash
# Check tool versions
rustc --version
node --version
yarn --version
circom --version  # Should be 2.1.9
forge --version
bazel --version

# Check environment variables
echo $TACHYON_DIR
echo $NODE_OPTIONS

# Check if Tachyon binary exists
ls -la $TACHYON_DIR/vendors/circom/bazel-bin/circomlib/build/compile_witness_generator
```

## Project Structure

```
.
├── circom/           # Main circom implementation
├── sdk-utils/        # Shared utilities
├── noir/            # Noir implementation (alternative)
├── Cargo.toml       # Workspace configuration
└── README.md        # This file
```

## Environment Requirements

- Ubuntu 20.04+ (or compatible Linux distribution)
- 8GB+ RAM (16GB+ recommended for large circuits)
- 10GB+ free disk space
- Internet connection for downloading dependencies

## License

[Add your license information here]