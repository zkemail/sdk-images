# Base image
FROM ubuntu:jammy AS noir-toolchain

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

# Install the matching Noir/Barretenberg releases directly. Moving installer
# branches can change independently of these versions.
RUN mkdir -p /root/.nargo/bin /root/.bb && \
    curl -fLsS https://github.com/noir-lang/noir/releases/download/v1.0.0-beta.5/nargo-x86_64-unknown-linux-gnu.tar.gz -o /tmp/nargo.tar.gz && \
    echo "a99c2aaab24e97a41a72224dd345aa82e8c8c21911b3d51ab38c232987fbdb5e  /tmp/nargo.tar.gz" | sha256sum -c - && \
    tar -xzf /tmp/nargo.tar.gz -C /root/.nargo/bin && \
    curl -fLsS https://github.com/AztecProtocol/aztec-packages/releases/download/v0.84.0/barretenberg-amd64-linux.tar.gz -o /tmp/bb.tar.gz && \
    echo "770b17b0c383923bd0ad5268c0440d6d472971e1e39a77f3a57523d299435c6e  /tmp/bb.tar.gz" | sha256sum -c - && \
    tar -xzf /tmp/bb.tar.gz -C /root/.bb && \
    rm /tmp/nargo.tar.gz /tmp/bb.tar.gz
ENV PATH="/root/.nargo/bin:$PATH"
ENV PATH="/root/.bb:$PATH"
RUN nargo --version && bb --version

FROM noir-toolchain AS runtime

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
