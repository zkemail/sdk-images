# Use the official Rust image as the base
FROM rust:latest

# Install necessary packages
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential

WORKDIR /

ENV NVM_DIR=/usr/local/nvm
ENV NODE_VERSION=22

RUN mkdir -p /usr/local/nvm

# Install nvm with node and npm
RUN curl https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash \
    && . $NVM_DIR/nvm.sh \
    && nvm install $NODE_VERSION \
    && nvm alias default $NODE_VERSION \
    && nvm use default

ENV NODE_PATH=$NVM_DIR/v$NODE_VERSION/lib/node_modules
ENV PATH=$NVM_DIR/v$NODE_VERSION/bin:$PATH

RUN npm install -g yarn

COPY . .

RUN cargo build --release

CMD ["./target/release/generate"]
