# Base image
FROM ubuntu:jammy

ENV DEBIAN_FRONTEND=noninteractive

# Update and install basic dependencies
RUN apt update && \
    apt install -y --no-install-recommends \
    wget \
    ca-certificates \
    curl \
    git \
    build-essential \
    gzip \
    zip && \
    rm -rf /var/lib/apt/lists/*

# Install Noir
RUN curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash
RUN noirup

# Install Barrentenberg
RUN curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
RUN bbup

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Node.js and Yarn
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt install -y nodejs && \
    npm install -g yarn snarkjs

# Install Foundry
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH="/root/.foundry/bin:$PATH"
RUN foundryup

# Final command
CMD ["cargo", "run"]