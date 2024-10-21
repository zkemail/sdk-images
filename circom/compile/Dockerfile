# Use the official Rust image as the base
FROM rust:latest

# Install necessary packages
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential

# Clone the Circom repository
RUN git clone https://github.com/iden3/circom.git /circom

# Build Circom
WORKDIR /circom
RUN cargo build --release

# Install Circom globally
RUN cargo install --path circom

# Add Cargo bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /

ENV NVM_DIR=/usr/local/nvm
ENV NODE_VERSION=22

RUN mkdir -p /usr/local/nvm

# Install nvm with node and npm
RUN curl https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash \
    && . $NVM_DIR/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && nvm use default \
    && npm install -g yarn

ENV NODE_PATH=$NVM_DIR/v$NODE_VERSION/lib/node_modules
ENV PATH=$NVM_DIR/v$NODE_VERSION/bin:$PATH

COPY Cargo.* ./

COPY src ./src

RUN cargo build --release

CMD ["./target/release/circom-compile"]
