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
    pkg-config \
    libssl-dev \
    zip \
    protobuf-compiler \
    libprotobuf-dev \
    && \
    rm -rf /var/lib/apt/lists/*

# Install Noir
RUN curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash
ENV PATH="/root/.nargo/bin:$PATH"
RUN noirup --version 1.0.0-beta.11

# Install Barrentenberg
RUN curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
ENV PATH="/root/.bb:$PATH"
RUN bbup --version 0.87.0

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

WORKDIR /app

COPY . .

# Verify protoc is installed
RUN protoc --version

# Build the project
RUN cargo build --bin noir --release

# Expose the port that the application listens on
EXPOSE 8082

WORKDIR /app/noir

ENV LOG_LEVEL=info

# Final command
CMD ["../target/release/noir"]
