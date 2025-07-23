# Base image
FROM ubuntu:jammy

# Arguments
ARG PYTHON_VERSION=3.10.12

ENV DEBIAN_FRONTEND=noninteractive

# Update and install basic dependencies
RUN apt update && \
    apt install -y --no-install-recommends \
    wget \
    ca-certificates \
    curl \
    git \
    build-essential \
    libbz2-dev \
    liblzma-dev \
    libncursesw5-dev \
    libreadline-dev \
    libsqlite3-dev \
    libssl-dev \
    make \
    python3-pip \
    python-is-python3 \
    tk-dev \
    xz-utils \
    zlib1g-dev \
    libgmp-dev \
    libomp-dev \
    nasm \
    nlohmann-json3-dev \
    gzip \
    protobuf-compiler \
    libprotobuf-dev \
    zip \
    unzip && \
    rm -rf /var/lib/apt/lists/*

# Install Python version using pyenv
RUN curl https://pyenv.run | bash
ENV HOME /root
RUN CONFIGURE_OPTS=--enable-shared $HOME/.pyenv/bin/pyenv install ${PYTHON_VERSION}
ENV PATH="${HOME}/.pyenv/shims:${HOME}/.pyenv/bin:${HOME}/bin:${PATH}"

# Install Bazel 6.3.0 for building Circom dependencies (required by tachyon)
RUN wget https://github.com/bazelbuild/bazel/releases/download/6.3.0/bazel-6.3.0-installer-linux-x86_64.sh && \
    chmod +x bazel-6.3.0-installer-linux-x86_64.sh && \
    ./bazel-6.3.0-installer-linux-x86_64.sh --user && \
    cp /root/bin/bazel /usr/local/bin/bazel && \
    rm bazel-6.3.0-installer-linux-x86_64.sh

# Clone and set up Tachyon
WORKDIR /root
RUN git clone https://github.com/kroma-network/tachyon.git
WORKDIR /root/tachyon

# Build Circom dependencies with Bazel
RUN cd vendors/circom && CARGO_BAZEL_REPIN=true bazel build --config opt //circomlib/build:compile_witness_generator

ENV TACHYON_DIR=/root/tachyon

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

# Clone and build Circom
RUN git clone https://github.com/iden3/circom.git /circom
WORKDIR /circom
RUN git checkout tags/v2.2.2 -b v2.2.2
RUN cargo build --release && cargo install --path circom

WORKDIR /root/circom

# Copy project files and build Tachyon
COPY . .
RUN cargo build --bin circom --release

# Set Node.js memory limits
ENV NODE_OPTIONS=--max_old_space_size=65536

RUN sysctl -w vm.max_map_count=262144

WORKDIR /root/circom/circom

ENV LOG_LEVEL=info

CMD ["../target/release/circom"]
